#!/bin/bash
while read image
do
    ../bin/dimgen -s -i $image
done < ./images.list
