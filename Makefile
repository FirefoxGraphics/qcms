
#COVERAGE_FLAGS=-fprofile-arcs -ftest-coverage 
#COVERAGE_FLAGS=
OPT_FLAGS=-O0
OPT_FLAGS=
CFLAGS=-Wall $(OPT_FLAGS) $(COVERAGE_FLAGS) -Wdeclaration-after-statement -ggdb `pkg-config --cflags lcms`
LDFLAGS=`pkg-config --libs lcms` -ldl

QCMS_SRC=iccread.c transform.c matrix.c chain.c transform_util.c
QCMS_OBJS=iccread.o transform.o matrix.o chain.o transform_util.o

PROGRAMS=profile-gen test test-invalid lcms-compare dump-profile div-test coverage malloc-fail invalid-coverage

all: $(PROGRAMS)

$(PROGRAMS): $(QCMS_OBJS)

clean:
	rm -f $(PROGRAMS) $(QCMS_OBJS)
