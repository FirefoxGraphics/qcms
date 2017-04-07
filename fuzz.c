#include <stdlib.h>
#include "qcms.h"

int main()
{
	unsigned char outt[3];
#define LENGTH 256*256*256
	static unsigned char src[LENGTH*3];
	static unsigned char output[LENGTH*3];
	int i,j,k,l=0;
	qcms_profile *input_profile, *output_profile;
	qcms_transform *transform;
	input_profile = qcms_profile_from_path("lcms_test/input.icc");
	output_profile = qcms_profile_from_path("lcms_test/output.icc");

	transform = qcms_transform_create(input_profile, QCMS_DATA_RGB_8, output_profile, QCMS_DATA_RGB_8, QCMS_INTENT_PERCEPTUAL);

	for (i=0; i<256; i++) {
		for (j=0; j<256; j++) {
			for (k=0; k<256; k++) {
				src[l++] = i;
				src[l++] = j;
				src[l++] = k;
			}
		}
	}

	qcms_transform_data(transform, src, output, LENGTH);

	qcms_transform_release(transform);
	qcms_profile_release(input_profile);
	qcms_profile_release(output_profile);

	return 0;
}
