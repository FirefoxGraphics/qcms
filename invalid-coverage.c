#include <stdlib.h>
#include <assert.h>
#include <stdint.h>
#include "qcms.h"

typedef uint32_t __be32;

/* __builtin_bswap isn't available in older gccs
 * so open code it for now */
static __be32 cpu_to_be32(int32_t v)
{
#ifdef LITTLE_ENDIAN
	return ((v & 0xff) << 24) | ((v & 0xff00) << 8) | ((v & 0xff0000) >> 8) | ((v & 0xff000000) >> 24);
	//return __builtin_bswap32(v);
#else
	return v;
#endif
}

char *buf;

static void write_u32(size_t offset, uint32_t value)
{
	*(__be32*)(buf + offset) = cpu_to_be32(value);
}
#if 0
static uint16_t read_u16(struct mem_source *mem, size_t offset)
{
	if (offset + 2 > mem->size) {
		invalid_source(mem, "Invalid offset");
		return 0;
	} else {
		return be16_to_cpu(*(__be16*)(mem->buf + offset));
	}
}
#endif
static void write_u8(size_t offset, uint8_t value)
{
	*(uint8_t*)(buf + offset) = value;
}
static void write_u16(size_t offset, uint16_t value)
{
	*(uint8_t*)(buf + offset + 1) = value & 0xff;
	*(uint8_t*)(buf + offset) = (value>>8) & 0xff;
}


#define PROFILE_LENGTH 18000
int main()
{
	qcms_profile_release(qcms_profile_sRGB());

	buf = calloc(PROFILE_LENGTH, 1);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	// invalid size
	write_u32(0, PROFILE_LENGTH + 2500);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	// proper size
	write_u32(0, PROFILE_LENGTH);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

#define INPUT_DEVICE_PROFILE   0x73636e72 // 'scnr'
	write_u32(12, INPUT_DEVICE_PROFILE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(8, 0x3); // invalid major revision
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(8, 0x2); // major revision
	write_u8(9, 0x55); // invalid minor revision
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(8, 0x2); // major revision
	write_u8(9, 0x40); // minor revision
	write_u8(10, 1); // reserved 1
	write_u8(11, 0); // reserved 2
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(10, 0); // reserved 1

	write_u8(64, 0x32); // invalid rendering intent
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(64, 0); // invalid rendering intent

#define RGB_SIGNATURE  0x52474220
#define GRAY_SIGNATURE 0x47524159
	write_u32(16, 0xdeadbeef);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(16, RGB_SIGNATURE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
#define XYZ_SIGNATURE  0x58595A20
	write_u32(20, XYZ_SIGNATURE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128, 15000); // tag count
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128, 15); // tag count
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

     #define TAG_bXYZ 0x6258595a
     #define TAG_gXYZ 0x6758595a
     #define TAG_rXYZ 0x7258595a
     #define TAG_rTRC 0x72545243
     #define TAG_bTRC 0x62545243
     #define TAG_gTRC 0x67545243
     #define TAG_kTRC 0x6b545243
     #define TAG_A2B0 0x41324230
     #define TAG_B2A0 0x42324130
     #define TAG_CHAD 0x63686164

	write_u32(128 + 4, TAG_rXYZ); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4, 1000); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*1*3, TAG_gXYZ); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4*1*3 + 4, 1000); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*2*3, TAG_bXYZ); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4*2*3 + 4, 1000); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

#define XYZ_TYPE   0x58595a20 // 'XYZ '
#define CURVE_TYPE 0x63757276 // 'curv'
#define LUT16_TYPE 0x6d667432 // 'mft2'
#define LUT8_TYPE  0x6d667431 // 'mft1'
#define LUT_MAB_TYPE		0x6d414220 // 'mAB '
#define LUT_MBA_TYPE		0x6d424120 // 'mBA '
#define CHROMATIC_TYPE          0x73663332 // 'sf32'
#define PARAMETRIC_CURVE_TYPE   0x70617261 // 'para'
	write_u32(1000, XYZ_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*3*3, TAG_rTRC); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4*3*3 + 4, 1100); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*4*3, TAG_gTRC); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4*4*3 + 4, 1100); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*5*3, TAG_bTRC); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4*5*3 + 4, 1100); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*6*3, TAG_A2B0); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*6*3 + 4, 1200); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*7*3, TAG_CHAD); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4*7*3 + 4, 5000); // offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200, 5); // invalid lut type
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200, LUT8_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(1200 + 8, 9); // max clut size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(1200 + 10, 9); // max clut size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(1200 + 8, 3); // in_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(1200 + 9, 3); // proper out_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(1200 + 10, 1); // sane clut size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200, LUT16_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u16(1200 + 48, 3); // input_table_entries
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u16(1200 + 50, 3); // output_table_entries
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*6*3, TAG_A2B0); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*6*3, TAG_B2A0); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200, LUT_MBA_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200, LUT_MAB_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*6*3, TAG_A2B0); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(1200 + 8, 15); // > max in_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(1200 + 9, 15); // > max out_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(1200 + 9, 4); // long out_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(1200 + 8, 2); // short in_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(1200 + 9, 3); // proper out_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(1200 + 8, 3); // proper out_chan
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));


	write_u32(1200 + 12, 5); // b curve offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200 + 12, 2000); // b curve offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200 + 16, 2000); // matrix offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200 + 20, 2000); // m curve offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200 + 24, 3000); // clut offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(4200, 255); // clut channel size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(4201, 255); // clut channel size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(4202, 255); // clut channel size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(4200, 2); // clut channel size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(4201, 2); // clut channel size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(4202, 2); // clut channel size
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u8(4216, 1); // clut precision
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u8(4216, 2); // clut precision
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1200 + 28, 2000); // a curve offset
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(3200, PARAMETRIC_CURVE_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u16(3208, 5);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u16(3208, 4);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u16(3208, 2);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(3200, CURVE_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	// reset the values
	write_u16(3208, 0);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));


	write_u32(3212, CURVE_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(3224, CURVE_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(5000, CHROMATIC_TYPE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
#if 1
	write_u32(1100, CURVE_TYPE);
	qcms_profile_release(qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1108, 100000); // curve count
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(1108, 1); // curve count
	qcms_profile_release(qcms_profile_from_memory(buf, PROFILE_LENGTH));

	/* test out gray profiles */
	write_u32(16, GRAY_SIGNATURE);
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));

	write_u32(128 + 4 + 4*8*3, TAG_kTRC); // tag
	assert(!qcms_profile_from_memory(buf, PROFILE_LENGTH));
	write_u32(128 + 4 + 4*8*3 + 4, 1100); // offset
	qcms_profile_release(qcms_profile_from_memory(buf, PROFILE_LENGTH));

	/* test out profiles that are the wrong size */
	qcms_profile_from_path("sample-trunc.icc");
#endif
	return 0;
}

