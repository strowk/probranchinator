# VHS recordings

This folder is used to record how probranchinator is used. 
For recordings [vhs](https://github.com/charmbracelet/vhs/) tool is used
through docker container (for stability and reproducibility).

## How to record

Firstly need to build docker image:

```bash
docker build -t vhs-probranchinator .
```

Then run the container:

```bash

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
```

This will start the container and run `base.tape` script inside it.

Alternatively can shell into the container and run the script manually:

```bash
docker run --rm -ti --privileged -v ${CURRENT_FOLDER}:/vhs  --entrypoint bash vhs-probranchinator
```

## Compare with baseline

Run "./compare.sh" script to make another recording and compare it with the baseline.