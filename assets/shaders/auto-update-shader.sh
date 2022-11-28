#!/bin/bash

for (( ; ; ))
do
    sed -E 's/0\.5/0\.4/g' -i pixelate.wgsl
    touch pixelate.wgsl
    sleep 5

    sed -E 's/0\.4/0\.5/g' -i pixelate.wgsl
    touch pixelate.wgsl
    sleep 5
done
