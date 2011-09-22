#include <stdlib.h>
#include <time.h>
#include "sum.h"
#include "lcms.h"
#include "qcms.h"

int main(int argc, char **argv)
{
	char *input_path = argv[1];
	char *output_path = argv[2];

	qcms_profile *input_profile, *output_profile;
	qcms_transform *transform;
#define LENGTH (256*256*256)
	static unsigned char src[LENGTH*3];
	static unsigned char qoutput[LENGTH*3];

	int i,j,k,l=0;
	for (i=0; i<256; i++) {
		for (j=0; j<256; j++) {
			for (k=0; k<256; k++) {
				src[l++] = i;
				src[l++] = j;
				src[l++] = k;
			}
		}
	}
	input_profile = qcms_profile_from_path(input_path);
	output_profile = qcms_profile_from_path(output_path);
	qcms_profile_precache_output_transform(output_profile);

	transform = qcms_transform_create(input_profile, QCMS_DATA_RGB_8, output_profile, QCMS_DATA_RGB_8, QCMS_INTENT_PERCEPTUAL);
	qcms_transform_data(transform, src, qoutput, LENGTH);
	qcms_profile_release(input_profile);
	qcms_profile_release(output_profile);
	return 0;
}
