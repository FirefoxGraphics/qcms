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


int main()
{
	qcms_profile_release(qcms_profile_sRGB());

	buf = calloc(1500, 1);
	assert(!qcms_profile_from_memory(buf, 1500));
	// invalid size
	write_u32(0, 2500);
	assert(!qcms_profile_from_memory(buf, 1500));

	// proper size
	write_u32(0, 1500);
	assert(!qcms_profile_from_memory(buf, 1500));

#define INPUT_DEVICE_PROFILE   0x73636e72 // 'scnr'
	write_u32(12, INPUT_DEVICE_PROFILE);
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u8(8, 0x3); // invalid major revision
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u8(8, 0x2); // major revision
	write_u8(9, 0x55); // invalid minor revision
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u8(8, 0x2); // major revision
	write_u8(9, 0x40); // minor revision
	write_u8(10, 1); // reserved 1
	write_u8(11, 0); // reserved 2
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u8(10, 0); // reserved 1

	write_u8(64, 0x32); // invalid rendering intent
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u8(64, 0); // invalid rendering intent

#define RGB_SIGNATURE  0x52474220
#define GRAY_SIGNATURE 0x47524159
	write_u32(16, RGB_SIGNATURE);
	assert(!qcms_profile_from_memory(buf, 1500));

#define XYZ_SIGNATURE  0x58595A20
	write_u32(20, XYZ_SIGNATURE);
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128, 15000); // tag count
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128, 15); // tag count
	assert(!qcms_profile_from_memory(buf, 1500));

     #define TAG_bXYZ 0x6258595a
     #define TAG_gXYZ 0x6758595a
     #define TAG_rXYZ 0x7258595a
     #define TAG_rTRC 0x72545243
     #define TAG_bTRC 0x62545243
     #define TAG_gTRC 0x67545243
     #define TAG_kTRC 0x6b545243
     #define TAG_A2B0 0x41324230

	write_u32(128 + 4, TAG_rXYZ); // tag
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u32(128 + 4 + 4, 1000); // offset
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128 + 4 + 4*1*3, TAG_gXYZ); // tag
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u32(128 + 4 + 4*1*3 + 4, 1000); // offset
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128 + 4 + 4*2*3, TAG_bXYZ); // tag
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u32(128 + 4 + 4*2*3 + 4, 1000); // offset
	assert(!qcms_profile_from_memory(buf, 1500));

#define XYZ_TYPE   0x58595a20 // 'XYZ '
#define CURVE_TYPE 0x63757276 // 'curv'
#define LUT16_TYPE 0x6d667432 // 'mft2'
#define LUT8_TYPE  0x6d667431 // 'mft1'

	write_u32(1000, XYZ_TYPE);
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128 + 4 + 4*3*3, TAG_rTRC); // tag
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u32(128 + 4 + 4*3*3 + 4, 1100); // offset
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128 + 4 + 4*4*3, TAG_gTRC); // tag
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u32(128 + 4 + 4*4*3 + 4, 1100); // offset
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128 + 4 + 4*5*3, TAG_bTRC); // tag
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u32(128 + 4 + 4*5*3 + 4, 1100); // offset
	assert(!qcms_profile_from_memory(buf, 1500));


	write_u32(1100, CURVE_TYPE);
	qcms_profile_release(qcms_profile_from_memory(buf, 1500));

	write_u32(1108, 100000); // curve count
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(1108, 1); // curve count
	qcms_profile_release(qcms_profile_from_memory(buf, 1500));

	/* test out gray profiles */
	write_u32(16, GRAY_SIGNATURE);
	assert(!qcms_profile_from_memory(buf, 1500));

	write_u32(128 + 4 + 4*6*3, TAG_kTRC); // tag
	assert(!qcms_profile_from_memory(buf, 1500));
	write_u32(128 + 4 + 4*6*3 + 4, 1100); // offset
	qcms_profile_release(qcms_profile_from_memory(buf, 1500));

	/* test out profiles that are the wrong size */
	qcms_profile_from_path("sample-trunc.icc");

	return 0;
}

