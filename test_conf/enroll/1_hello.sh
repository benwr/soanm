#!/usr/bin/env sh

echo "This is running in script enroll/1_hello.sh on the enrollee"
echo "This is running in enroll/1_hello.sh on the enrollee and being piped to stderr" 1>&2
echo "called as: " $@
echo "enroll/1_hello.sh will pipe its input back to stdout." 1>&2
cat
