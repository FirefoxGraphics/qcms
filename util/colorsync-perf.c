#include <ApplicationServices/ApplicationServices.h>
#include <dlfcn.h>
#include <stdio.h>

#define BITMAP_INFO (kCGBitmapByteOrder32Big | kCGImageAlphaNoneSkipLast)


int main(int argc, char **argv) {
	int width = 256;
	int height = 256*256;
	char *input_profile_file = "input.icc";
	char *output_profile_file = "output.icc";
	if (argc >= 3) {
		input_profile_file = argv[1];
		output_profile_file = argv[2];
	}
	CGDataProviderRef input_file = CGDataProviderCreateWithFilename(input_profile_file);
	CGDataProviderRef output_file = CGDataProviderCreateWithFilename(output_profile_file);
	float range[] = {0, 1., 0, 1., 0, 1.};
	CGColorSpaceRef output_profile = CGColorSpaceCreateICCBased(3, range, output_file, NULL);
	CGColorSpaceRef input_profile = CGColorSpaceCreateICCBased(3, range, input_file, NULL);
	CGColorSpaceRef cs = CGColorSpaceCreateDeviceRGB();

	CGContextRef cin = CGBitmapContextCreate(NULL, width, height,
			8, 4*width, input_profile, BITMAP_INFO);
	CGContextRef cout = CGBitmapContextCreate(NULL, width, height,
			8, 4*width, output_profile, BITMAP_INFO);

	CGRect rect = {{0,0},{width, height}};
	CGImageRef copy = CGBitmapContextCreateImage(cin);
#define LENGTH 256*256*256
	static unsigned char src[LENGTH*3];
	static unsigned char qoutput[LENGTH*3];
	static unsigned char loutput[LENGTH*3];
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

	CGDataProviderRef dp = CGDataProviderCreateWithData(NULL, src, height * width * 3, NULL);
	CGImageRef ref = CGImageCreate(width, height, 8, 24, width * 3, input_profile, BITMAP_INFO, dp, NULL, 1, kCGRenderingIntentDefault);
	clock_t qcms_start = clock();
	CGContextDrawImage(cout, rect, ref);
	clock_t qcms_time = clock() - qcms_start;
	printf("ColorSync: %ld\n", qcms_time);
}
