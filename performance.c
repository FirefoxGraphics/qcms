#include <stdlib.h>
#include <time.h>
#include "sum.h"
#include "qcms.h"

int main(int argc, char **argv)
{
	char *input_path = argv[1];
	char *output_path = argv[2];
	
	qcms_profile *input_profile, *output_profile;
	qcms_transform *transform;
#define ALL
#ifndef ALL
#define LENGTH 1
#else
#define LENGTH (256*256*256)
#endif
	static unsigned char src[LENGTH*3];
	static unsigned char qoutput[LENGTH*3];
	static unsigned char loutput[LENGTH*3];
#ifdef ALL
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
#else
	int i;
	src[0] = 19;
	src[1] = 28;
	src[2] = 56;
#endif
	clock_t qcms_start = clock();
	input_profile = qcms_profile_from_path(input_path);
	output_profile = qcms_profile_from_path(output_path);
	qcms_profile_precache_output_transform(output_profile);

	transform = qcms_transform_create(input_profile, QCMS_DATA_RGB_8, output_profile, QCMS_DATA_RGB_8, QCMS_INTENT_PERCEPTUAL);
	qcms_transform_data(transform, src, qoutput, LENGTH);
	clock_t qcms_time = clock() - qcms_start;
	printf("qcms: %ld\n", qcms_time);
	int total_diff = 0;
	int diff_sum = 0;
	for (i=0; i<LENGTH; i++) {
		int diff = 0;
		diff_sum += (loutput[i*3]-qoutput[i*3]);
		diff_sum += (loutput[i*3+1]-qoutput[i*3+1]);
		diff_sum += (loutput[i*3+2]-qoutput[i*3+2]);
		diff += abs(loutput[i*3]-qoutput[i*3]);
		diff += abs(loutput[i*3+1]-qoutput[i*3+1]);
		diff += abs(loutput[i*3+2]-qoutput[i*3+2]);
		total_diff += diff;
	}
	printf("%d %d - %f\n", diff_sum, total_diff, (double)total_diff/LENGTH);
	qcms_profile_release(input_profile);
	qcms_profile_release(output_profile);
	return 0;
}
