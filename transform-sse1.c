//  qcms
//  Copyright (C) 2009 Mozilla Foundation
//  Copyright (C) 2010 Steve Snyder
//
// Permission is hereby granted, free of charge, to any person obtaining 
// a copy of this software and associated documentation files (the "Software"), 
// to deal in the Software without restriction, including without limitation 
// the rights to use, copy, modify, merge, publish, distribute, sublicense, 
// and/or sell copies of the Software, and to permit persons to whom the Software 
// is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in 
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, 
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO 
// THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND 
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE 
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION 
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION 
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#include <xmmintrin.h>

#include "qcmsint.h"

/* pre-shuffled: just load these into XMM reg instead of load-scalar/shufps sequence */
#define FLOATSCALE  (float)(PRECACHE_OUTPUT_SIZE)
#define CLAMPMAXVAL ( ((float) (PRECACHE_OUTPUT_SIZE - 1)) / PRECACHE_OUTPUT_SIZE )
static const ALIGN float floatScaleX4[4] =
    { FLOATSCALE, FLOATSCALE, FLOATSCALE, FLOATSCALE};
static const ALIGN float clampMaxValueX4[4] =
    { CLAMPMAXVAL, CLAMPMAXVAL, CLAMPMAXVAL, CLAMPMAXVAL};

void qcms_transform_data_rgb_out_lut_sse1(qcms_transform *transform,
                                          unsigned char *src,
                                          unsigned char *dest,
                                          size_t length)
{
    unsigned int i;
    float (*mat)[4] = transform->matrix;
    char input_back[32];
    /* Ensure we have a buffer that's 16 byte aligned regardless of the original
     * stack alignment. We can't use __attribute__((aligned(16))) or __declspec(align(32))
     * because they don't work on stack variables. gcc 4.4 does do the right thing
     * on x86 but that's too new for us right now. For more info: gcc bug #16660 */
    float const * input = (float*)(((uintptr_t)&input_back[16]) & ~0xf);
    /* share input and output locations to save having to keep the
     * locations in separate registers */
    uint32_t const * output = (uint32_t*)input;

    /* deref *transform now to avoid it in loop */
    const float *igtbl_r = transform->input_gamma_table_r;
    const float *igtbl_g = transform->input_gamma_table_g;
    const float *igtbl_b = transform->input_gamma_table_b;

    /* deref *transform now to avoid it in loop */
    const uint8_t *otdata_r = &transform->output_table_r->data[0];
    const uint8_t *otdata_g = &transform->output_table_g->data[0];
    const uint8_t *otdata_b = &transform->output_table_b->data[0];

    /* input matrix values never change */
    const __m128 mat0  = _mm_load_ps(mat[0]);
    const __m128 mat1  = _mm_load_ps(mat[1]);
    const __m128 mat2  = _mm_load_ps(mat[2]);

    /* these values don't change, either */
    const __m128 max   = _mm_load_ps(clampMaxValueX4);
    const __m128 min   = _mm_setzero_ps();
    const __m128 scale = _mm_load_ps(floatScaleX4);

    /* working variables */
    __m128 vec_r, vec_g, vec_b, result;

    /* CYA */
    if (!length)
        return;

    /* one pixel is handled outside of the loop */
    length--;

    /* setup for transforming 1st pixel */
    vec_r = _mm_load_ss(&igtbl_r[src[0]]);
    vec_g = _mm_load_ss(&igtbl_g[src[1]]);
    vec_b = _mm_load_ss(&igtbl_b[src[2]]);
    src += 3;

    /* transform all but final pixel */

    for (i=0; i<length; i++)
    {
        /* position values from gamma tables */
        vec_r = _mm_shuffle_ps(vec_r, vec_r, 0);
        vec_g = _mm_shuffle_ps(vec_g, vec_g, 0);
        vec_b = _mm_shuffle_ps(vec_b, vec_b, 0);

        /* gamma * matrix */
        vec_r = _mm_mul_ps(vec_r, mat0);
        vec_g = _mm_mul_ps(vec_g, mat1);
        vec_b = _mm_mul_ps(vec_b, mat2);

        /* crunch, crunch, crunch */
        vec_r  = _mm_add_ps(vec_r, _mm_add_ps(vec_g, vec_b));
        vec_r  = _mm_max_ps(min, vec_r);
        vec_r  = _mm_min_ps(max, vec_r);
        result = _mm_mul_ps(vec_r, scale);

        /* store calc'd output tables indices */
        *((__m64 *)&output[0]) = _mm_cvtps_pi32(result);
        result = _mm_movehl_ps(result, result);
        *((__m64 *)&output[2]) = _mm_cvtps_pi32(result) ;

        /* load for next loop while store completes */
        vec_r = _mm_load_ss(&igtbl_r[src[0]]);
        vec_g = _mm_load_ss(&igtbl_g[src[1]]);
        vec_b = _mm_load_ss(&igtbl_b[src[2]]);
        src += 3;

        /* use calc'd indices to output RGB values */
        dest[OUTPUT_R_INDEX] = otdata_r[output[0]];
        dest[OUTPUT_G_INDEX] = otdata_g[output[1]];
        dest[OUTPUT_B_INDEX] = otdata_b[output[2]];
        dest += RGB_OUTPUT_COMPONENTS;
    }

    /* handle final (maybe only) pixel */

    vec_r = _mm_shuffle_ps(vec_r, vec_r, 0);
    vec_g = _mm_shuffle_ps(vec_g, vec_g, 0);
    vec_b = _mm_shuffle_ps(vec_b, vec_b, 0);

    vec_r = _mm_mul_ps(vec_r, mat0);
    vec_g = _mm_mul_ps(vec_g, mat1);
    vec_b = _mm_mul_ps(vec_b, mat2);

    vec_r  = _mm_add_ps(vec_r, _mm_add_ps(vec_g, vec_b));
    vec_r  = _mm_max_ps(min, vec_r);
    vec_r  = _mm_min_ps(max, vec_r);
    result = _mm_mul_ps(vec_r, scale);

    *((__m64 *)&output[0]) = _mm_cvtps_pi32(result);
    result = _mm_movehl_ps(result, result);
    *((__m64 *)&output[2]) = _mm_cvtps_pi32(result);

    dest[OUTPUT_R_INDEX] = otdata_r[output[0]];
    dest[OUTPUT_G_INDEX] = otdata_g[output[1]];
    dest[OUTPUT_B_INDEX] = otdata_b[output[2]];

    _mm_empty();
}

void qcms_transform_data_rgba_out_lut_sse1(qcms_transform *transform,
                                           unsigned char *src,
                                           unsigned char *dest,
                                           size_t length)
{
    unsigned int i;
    float (*mat)[4] = transform->matrix;
    char input_back[32];
    /* Ensure we have a buffer that's 16 byte aligned regardless of the original
     * stack alignment. We can't use __attribute__((aligned(16))) or __declspec(align(32))
     * because they don't work on stack variables. gcc 4.4 does do the right thing
     * on x86 but that's too new for us right now. For more info: gcc bug #16660 */
    float const * input = (float*)(((uintptr_t)&input_back[16]) & ~0xf);
    /* share input and output locations to save having to keep the
     * locations in separate registers */
    uint32_t const * output = (uint32_t*)input;

    /* deref *transform now to avoid it in loop */
    const float *igtbl_r = transform->input_gamma_table_r;
    const float *igtbl_g = transform->input_gamma_table_g;
    const float *igtbl_b = transform->input_gamma_table_b;

    /* deref *transform now to avoid it in loop */
    const uint8_t *otdata_r = &transform->output_table_r->data[0];
    const uint8_t *otdata_g = &transform->output_table_g->data[0];
    const uint8_t *otdata_b = &transform->output_table_b->data[0];

    /* input matrix values never change */
    const __m128 mat0  = _mm_load_ps(mat[0]);
    const __m128 mat1  = _mm_load_ps(mat[1]);
    const __m128 mat2  = _mm_load_ps(mat[2]);

    /* these values don't change, either */
    const __m128 max   = _mm_load_ps(clampMaxValueX4);
    const __m128 min   = _mm_setzero_ps();
    const __m128 scale = _mm_load_ps(floatScaleX4);

    /* working variables */
    __m128 vec_r, vec_g, vec_b, result;
    unsigned char alpha;

    /* CYA */
    if (!length)
        return;

    /* one pixel is handled outside of the loop */
    length--;

    /* setup for transforming 1st pixel */
    vec_r = _mm_load_ss(&igtbl_r[src[0]]);
    vec_g = _mm_load_ss(&igtbl_g[src[1]]);
    vec_b = _mm_load_ss(&igtbl_b[src[2]]);
    alpha = src[3];
    src += 4;

    /* transform all but final pixel */

    for (i=0; i<length; i++)
    {
        /* position values from gamma tables */
        vec_r = _mm_shuffle_ps(vec_r, vec_r, 0);
        vec_g = _mm_shuffle_ps(vec_g, vec_g, 0);
        vec_b = _mm_shuffle_ps(vec_b, vec_b, 0);

        /* gamma * matrix */
        vec_r = _mm_mul_ps(vec_r, mat0);
        vec_g = _mm_mul_ps(vec_g, mat1);
        vec_b = _mm_mul_ps(vec_b, mat2);

        /* store alpha for this pixel; load alpha for next */
        dest[OUTPUT_A_INDEX] = alpha;
        alpha   = src[3];

        /* crunch, crunch, crunch */
        vec_r  = _mm_add_ps(vec_r, _mm_add_ps(vec_g, vec_b));
        vec_r  = _mm_max_ps(min, vec_r);
        vec_r  = _mm_min_ps(max, vec_r);
        result = _mm_mul_ps(vec_r, scale);

        /* store calc'd output tables indices */
        *((__m64 *)&output[0]) = _mm_cvtps_pi32(result);
        result = _mm_movehl_ps(result, result);
        *((__m64 *)&output[2]) = _mm_cvtps_pi32(result);

        /* load gamma values for next loop while store completes */
        vec_r = _mm_load_ss(&igtbl_r[src[0]]);
        vec_g = _mm_load_ss(&igtbl_g[src[1]]);
        vec_b = _mm_load_ss(&igtbl_b[src[2]]);
        src += 4;

        /* use calc'd indices to output RGB values */
        dest[OUTPUT_R_INDEX] = otdata_r[output[0]];
        dest[OUTPUT_G_INDEX] = otdata_g[output[1]];
        dest[OUTPUT_B_INDEX] = otdata_b[output[2]];
        dest += 4;
    }

    /* handle final (maybe only) pixel */

    vec_r = _mm_shuffle_ps(vec_r, vec_r, 0);
    vec_g = _mm_shuffle_ps(vec_g, vec_g, 0);
    vec_b = _mm_shuffle_ps(vec_b, vec_b, 0);

    vec_r = _mm_mul_ps(vec_r, mat0);
    vec_g = _mm_mul_ps(vec_g, mat1);
    vec_b = _mm_mul_ps(vec_b, mat2);

    dest[OUTPUT_A_INDEX] = alpha;

    vec_r  = _mm_add_ps(vec_r, _mm_add_ps(vec_g, vec_b));
    vec_r  = _mm_max_ps(min, vec_r);
    vec_r  = _mm_min_ps(max, vec_r);
    result = _mm_mul_ps(vec_r, scale);

    *((__m64 *)&output[0]) = _mm_cvtps_pi32(result);
    result = _mm_movehl_ps(result, result);
    *((__m64 *)&output[2]) = _mm_cvtps_pi32(result);

    dest[OUTPUT_R_INDEX] = otdata_r[output[0]];
    dest[OUTPUT_G_INDEX] = otdata_g[output[1]];
    dest[OUTPUT_B_INDEX] = otdata_b[output[2]];

    _mm_empty();
}


#define ONE_SHIFT 14
static const ALIGN float fixScaleX4[4] =
    { 1<<ONE_SHIFT, 1<<ONE_SHIFT, 1<<ONE_SHIFT, 1<<ONE_SHIFT};

void qcms_transform_data_rgb_out_lut_mmx(qcms_transform *transform,
                                          unsigned char *src,
                                          unsigned char *dest,
                                          size_t length)
{
    unsigned int i;
    int (*mat)[4] = transform->matrix;
    char input_back[32];
    /* Ensure we have a buffer that's 16 byte aligned regardless of the original
     * stack alignment. We can't use __attribute__((aligned(16))) or __declspec(align(32))
     * because they don't work on stack variables. gcc 4.4 does do the right thing
     * on x86 but that's too new for us right now. For more info: gcc bug #16660 */
    float const * input = (float*)(((uintptr_t)&input_back[16]) & ~0xf);
    /* share input and output locations to save having to keep the
     * locations in separate registers */
    uint32_t const * output = (uint32_t*)input;

    /* deref *transform now to avoid it in loop */
    const int *igtbl_r = transform->input_gamma_table_r;
    const int *igtbl_g = transform->input_gamma_table_g;
    const int *igtbl_b = transform->input_gamma_table_b;

    /* deref *transform now to avoid it in loop */
    const uint8_t *otdata_r = &transform->output_table_r->data[0];
    const uint8_t *otdata_g = &transform->output_table_g->data[0];
    const uint8_t *otdata_b = &transform->output_table_b->data[0];

    /* these values don't change, either */
    const __m64 max   = _mm_set1_pi16(4096);
    const __m64 min   = _mm_setzero_si64();
    const __m64 half  = _mm_set1_pi32(1<<(ONE_SHIFT+(ONE_SHIFT-12)-1));

    /* input matrix values never change */
    const __m64 mat0_lo  = *(__m64*)&(mat[0][0]);
    const __m64 mat0_hi  = *(__m64*)&(mat[0][2]);
    const __m64 mat1_lo  = *(__m64*)&(mat[1][0]);
    const __m64 mat1_hi  = *(__m64*)&(mat[1][2]);
    const __m64 mat01_lo = _mm_unpacklo_pi16(_mm_packs_pi32(mat0_lo, mat0_lo), _mm_packs_pi32(mat1_lo, mat1_lo));
    const __m64 mat01_hi = _mm_unpacklo_pi16(_mm_packs_pi32(mat0_hi, mat0_hi), _mm_packs_pi32(mat1_hi, mat1_hi));
    const __m64 mat2_lo  = *(__m64*)&(mat[2][0]);
    const __m64 mat2_hi  = *(__m64*)&(mat[2][2]);

    /* working variables */
    __m64 vec_r_lo, vec_g_lo, vec_b_lo, result_lo;
    __m64 vec_r_hi, vec_g_hi, vec_b_hi, result_hi;

    /* CYA */
    if (!length)
        return;

    /* one pixel is handled outside of the loop */
    length--;

    /* setup for transforming 1st pixel */
    vec_r_lo = _mm_set1_pi16(igtbl_r[src[0]]);
    vec_r_hi = _mm_set1_pi16(igtbl_r[src[0]]);
    vec_g_lo = _mm_set1_pi16(igtbl_g[src[1]]);
    vec_g_hi = _mm_set1_pi16(igtbl_g[src[1]]);
    vec_b_lo = _mm_set1_pi32(igtbl_b[src[2]]);
    vec_b_hi = _mm_set1_pi32(igtbl_b[src[2]]);
    src += 3;

    /* transform all but final pixel */

    for (i=0; i<length; i++)
    {
        /* position values from gamma tables */
        __m64 vec_rg_lo, vec_rg_hi;
	vec_rg_lo = vec_rg_hi = _mm_unpacklo_pi16(vec_r_lo, vec_g_lo);
        /* gamma * matrix */
        vec_rg_lo = _mm_madd_pi16(vec_rg_lo, mat01_lo);
        vec_rg_hi = _mm_madd_pi16(vec_rg_hi, mat01_hi);
        vec_b_lo = _mm_madd_pi16(vec_b_lo, mat2_lo);
        vec_b_hi = _mm_madd_pi16(vec_b_hi, mat2_hi);

        __m64 vec_r1  = _mm_add_pi32(vec_b_lo, vec_rg_lo);
	vec_r1  = _mm_add_pi32(vec_r1, half);

	__m64 vec_r2  = _mm_add_pi32(vec_b_hi, vec_rg_hi);
	vec_r2  = _mm_add_pi32(vec_r2, half);

	vec_r1  = _mm_srai_pi32(vec_r1, ONE_SHIFT+(ONE_SHIFT-12));
        vec_r2  = _mm_srai_pi32(vec_r2, ONE_SHIFT+(ONE_SHIFT-12));
        vec_r1 = _mm_max_pi16(min, vec_r1);
        vec_r2 = _mm_max_pi16(min, vec_r2);
        result_lo = _mm_min_pi16(max, vec_r1);
        result_hi = _mm_min_pi16(max, vec_r2);

        /* store calc'd output tables indices */
        *((__m64*)output) = result_lo;
        *(__m64*)(output+2) =  result_hi;

        /* load for next loop while store completes */
	vec_r_hi = vec_r_lo = _mm_set1_pi16(igtbl_r[src[0]]);
	vec_g_hi = vec_g_lo = _mm_set1_pi16(igtbl_g[src[1]]);
	vec_b_hi = vec_b_lo = _mm_set1_pi32(igtbl_b[src[2]]);
	src += 3;

        /* use calc'd indices to output RGB values */
        dest[0] = otdata_r[output[0]];
        dest[1] = otdata_g[output[1]];
        dest[2] = otdata_b[output[2]];
        dest += 3;
    }

    /* handle final (maybe only) pixel */

    __m64 vec_rg_lo = _mm_unpacklo_pi16(vec_r_lo, vec_g_lo);
    __m64 vec_rg_hi = _mm_unpacklo_pi16(vec_r_hi, vec_g_hi);

    /* gamma * matrix */
    vec_rg_lo = _mm_madd_pi16(vec_rg_lo, mat01_lo);
    vec_rg_hi = _mm_madd_pi16(vec_rg_hi, mat01_hi);
    vec_b_lo = _mm_madd_pi16(vec_b_lo, mat2_lo);
    vec_b_hi = _mm_madd_pi16(vec_b_hi, mat2_hi);

    __m64 vec_r1  = _mm_add_pi32(vec_b_lo, vec_rg_lo);
    vec_r1  = _mm_add_pi32(vec_r1, half);

    __m64 vec_r2  = _mm_add_pi32(vec_b_hi, vec_rg_hi);
    vec_r2  = _mm_add_pi32(vec_r2, half);

    vec_r1  = _mm_srai_pi32(vec_r1, ONE_SHIFT+(ONE_SHIFT-12));
    vec_r2  = _mm_srai_pi32(vec_r2, ONE_SHIFT+(ONE_SHIFT-12));
    vec_r1 = _mm_max_pi16(min, vec_r1);
    vec_r2 = _mm_max_pi16(min, vec_r2);
    result_lo = _mm_min_pi16(max, vec_r1);
    result_hi = _mm_min_pi16(max, vec_r2);

    /* store calc'd output tables indices */
    *(__m64*)output = result_lo;
    *(__m64*)(output+2) =  result_hi;

    _mm_empty();

    dest[0] = otdata_r[output[0]];
    dest[1] = otdata_g[output[1]];
    dest[2] = otdata_b[output[2]];
}


