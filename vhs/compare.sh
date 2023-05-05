#!/bin/bash

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    CURRENT_FOLDER=$(pwd)
elif [[ "$OSTYPE" == "darwin"* ]]; then
    CURRENT_FOLDER=$(pwd)
elif [[ "$OSTYPE" == "cygwin" ]]; then
    CURRENT_FOLDER=$(pwd -W)
elif [[ "$OSTYPE" == "msys" ]]; then
    CURRENT_FOLDER=$(pwd -W)
elif [[ "$OSTYPE" == "win32" ]]; then
    CURRENT_FOLDER=$(pwd -W)
fi

docker run --rm -ti --privileged -v ${CURRENT_FOLDER}:/vhs vhs-probranchinator ./base.tape

if  ( diff new.txt base.txt ) then
    echo difference not found 
else
    echo difference found
    exit 1
fi

