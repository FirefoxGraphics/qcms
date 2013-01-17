
COVERAGE_FLAGS=-fprofile-arcs -ftest-coverage
COVERAGE_FLAGS=
OPT_FLAGS=
OPT_FLAGS=-O2
CFLAGS=`pkg-config --cflags lcms` -Wall $(OPT_FLAGS) $(COVERAGE_FLAGS) -Wdeclaration-after-statement -ggdb
LDFLAGS=-ldl

QCMS_SRC=iccread.c transform.c matrix.c chain.c transform-util.c transform-sse2.c transform-sse1.c
QCMS_OBJS=iccread.o transform.o matrix.o chain.o transform-util.o transform-sse2.o transform-sse1.o

PROGRAMS=performance profile-gen test test-invalid test-transform dump-profile div-test coverage malloc-fail invalid-coverage

# I don't know a good way to get the exit code of pkg-config into a make variable
HAS_LCMS:=$(shell pkg-config --exists lcms; echo $$?)
ifeq ($(HAS_LCMS),0)
PROGRAMS+=lcms-compare
CFLAGS+=`pkg-config --cflags lcms`
LDFLAGS+=`pkg-config --libs lcms`
endif

all: $(PROGRAMS)

$(PROGRAMS): $(QCMS_OBJS)

gen-coverage:
	mkdir -p lcov
	lcov -d . -c --output-file lcov/lcov.info
	genhtml -o lcov/ lcov/lcov.info

clean:
	rm -f $(PROGRAMS) $(QCMS_OBJS)
