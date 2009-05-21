#include <stdlib.h>
#include "qcms.h"

void dump_profile(qcms_profile *profile)
{
	if (profile) {
		printf(" rendering intent: %d\n", qcms_profile_get_rendering_intent(profile));
		qcms_profile_release(profile);
	} else {
		printf("bad profile\n");
	}
}

int main(int argc, char **argv)
{
	if (argc > 1) {
		int i;
		for (i=1; i<argc; i++) {
			char *input_path = argv[i];
			printf("%s\n", input_path);
			dump_profile(qcms_profile_from_path(input_path));
		}
	} else {
		dump_profile(qcms_profile_sRGB());
	}


	return 0;
}
