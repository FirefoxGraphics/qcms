#include <stdlib.h>
#include <assert.h>
#include "qcms.h"


int main()
{
	assert(!qcms_profile_from_path("bad-profiles/rand.icc"));
	assert(!qcms_profile_from_path("bad-profiles/curve.icc"));
	return 0;
}
