#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
uint32_t ref(uint32_t a)
{
	return (a + 128)/ 255;
	return (a + (65535/2))/ 65535;
}

uint32_t try1(a)
{
	return (((uint64_t)a + 128)*257) / 65535LL;
	return (((uint64_t)a + 32767)*65538) / 4294967296LL;
}



int main()
{
	uint32_t x = 0;
	do {
		uint32_t r1 = ref(x);
		uint32_t r2 = try1(x);
		if (r1 != r2) {
			printf("FAIL: %x -> %x %x\n", x, r1, r2);
			exit(1);
		}
		if (x == 0xffffffff)
			break;
		x++;
	} while (1);
	return 0;
}
