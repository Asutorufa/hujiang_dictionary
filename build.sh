function suffix() {
    if test $1 = "windows"; then
        echo ".exe"
    fi
}

OS="android darwin freebsd openbsd windows linux"
ARCH="mips mips64 386 arm arm64 amd64"
for os in $OS; do
    for arch in $ARCH; do
        echo hj_${os}_$arch$(suffix $os)
        CGO_ENABLED=0 GOOS=${os} GOARCH=${arch} go build -ldflags="-s -w -buildid=" -trimpath -o output/${os}/hj_${os}_${arch}$(suffix $os) ./cmd/hj/...
        echo google_${os}_$arch$(suffix $os)
        CGO_ENABLED=0 GOOS=${os} GOARCH=${arch} go build -ldflags="-s -w -buildid=" -trimpath -o output/${os}/google_${os}_${arch}$(suffix $os) ./cmd/google/...
    done
done
