#!/usr/bin/env bash
SCRIPT_DIR=$(dirname `realpath $0`)
OCIBUILDER="${SCRIPT_DIR}/../bin/ocibuilder"
if [ ! -f $OCIBUILDER ] ; then
    echo "cannot locate ocibuilder bin: $OCIBUILDER"
    exit 1
fi

cntr=$($OCIBUILDER from quay.io/quay/busybox:latest | tail -1)
$OCIBUILDER config --author navid --working-dir /tmp $cntr
$OCIBUILDER config --label "owner=ocibuilder" $cntr
$OCIBUILDER run $cntr touch ocibuilder.txt
$OCIBUILDER commit $cntr quay.io/ocibuilder/ocibuilder-test:latest
[ -f "/tmp/ocibuilder_new_image.tar" ] && /bin/rm -rf /tmp/ocibuilder_new_image.tar
$OCIBUILDER save -o /tmp/ocibuilder_new_image.tar quay.io/ocibuilder/ocibuilder-test:latest

exit 0
