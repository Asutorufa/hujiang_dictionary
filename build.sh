CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o output/hj_linux_amd64 main.go
CGO_ENABLED=0 GOOS=linux GOARCH=386 go build -o output/hj_linux_386 main.go
CGO_ENABLED=0 GOOS=linux GOARCH=arm go build -o output/hj_linux_arm main.go
CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -o output/hj_linux_arm64 main.go
CGO_ENABLED=0 GOOS=linux GOARCH=mips go build -o output/hj_linux_mips main.go
CGO_ENABLED=0 GOOS=linux GOARCH=mips64 go build -o output/hj_linux_mips64 main.go

CGO_ENABLED=0 GOOS=android GOARCH=386 go build -o output/hj_android_386 main.go
CGO_ENABLED=0 GOOS=android GOARCH=amd64 go build -o output/hj_android_amd64 main.go
CGO_ENABLED=0 GOOS=android GOARCH=arm go build -o output/hj_android_arm main.go
CGO_ENABLED=0 GOOS=android GOARCH=arm64 go build -o output/hj_android_arm64 main.go

CGO_ENABLED=0 GOOS=darwin GOARCH=386 go build -o output/hj_darwin_386 main.go
CGO_ENABLED=0 GOOS=darwin GOARCH=amd64 go build -o output/hj_darwin_amd64 main.go
CGO_ENABLED=0 GOOS=darwin GOARCH=arm go build -o output/hj_darwin_arm main.go
CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -o output/hj_darwin_arm64 main.go

CGO_ENABLED=0 GOOS=freebsd GOARCH=386 go build -o output/hj_freebsd_386 main.go
CGO_ENABLED=0 GOOS=freebsd GOARCH=amd64 go build -o output/hj_freebsd_amd64 main.go
CGO_ENABLED=0 GOOS=freebsd GOARCH=arm go build -o output/hj_freebsd_arm main.go

CGO_ENABLED=0 GOOS=openbsd GOARCH=386 go build -o output/hj_openbsd_386 main.go
CGO_ENABLED=0 GOOS=openbsd GOARCH=amd64 go build -o output/hj_openbsd_amd64 main.go
CGO_ENABLED=0 GOOS=openbsd GOARCH=arm go build -o output/hj_openbsd_arm main.go
CGO_ENABLED=0 GOOS=openbsd GOARCH=arm64 go build -o output/hj_openbsd_arm64 main.go

CGO_ENABLED=0 GOOS=windows GOARCH=386 go build -o output/hj_windows_386.exe main.go
CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -o output/hj_windows_amd64.exe main.go
CGO_ENABLED=0 GOOS=windows GOARCH=arm go build -o output/hj_windows_arm.exe main.go