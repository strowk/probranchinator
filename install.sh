#!/bin/bash

THESYSTEMIS="unknown-linux-gnu"
POSTFIX=""

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    THESYSTEMIS="unknown-linux-gnu"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    THESYSTEMIS="apple-darwin"
elif [[ "$OSTYPE" == "cygwin" ]]; then
    THESYSTEMIS="pc-windows-gnu"
elif [[ "$OSTYPE" == "msys" ]]; then
    THESYSTEMIS="pc-windows-gnu"
elif [[ "$OSTYPE" == "win32" ]]; then
    THESYSTEMIS="pc-windows-gnu"
fi

if [[ "$THESYSTEMIS" == "pc-windows-gnu" ]]; then
    POSTFIX=".exe"
fi

echo "The system is $THESYSTEMIS"
ARCH="$(uname -m)"
echo "architecture is $ARCH"

BUILD="${ARCH}-${THESYSTEMIS}"
DOWNLOAD_URL="$(curl https://api.github.com/repos/strowk/probranchinator/releases/latest | grep browser_download_url | grep ${BUILD} | cut -d '"' -f 4 )"

echo "Downloading from $DOWNLOAD_URL"
curl "$DOWNLOAD_URL" -Lo ./probranchinator-archive.tgz
mkdir -p ./probranchinator-install
tar -xzf ./probranchinator-archive.tgz -C ./probranchinator-install

INSTALL_SOURCE="./probranchinator-install/target/${BUILD}/release/probranchinator${POSTFIX}"
INSTALL_TARGET="/usr/local/bin/probranchinator"

chmod +x "${INSTALL_SOURCE}"

if [[ "$THESYSTEMIS" == "pc-windows-gnu" ]]; then
    mkdir -p /usr/local/bin
    mv "${INSTALL_SOURCE}" "${INSTALL_TARGET}${POSTFIX}"
else 
    sudo mv "${INSTALL_SOURCE}" "${INSTALL_TARGET}${POSTFIX}"
fi

rm probranchinator-install/ -r
rm probranchinator-archive.tgz

echo "Probranchinator is installed, checking by running 'probranchinator --version'"
probranchinator --version