CXX      ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -Wpedantic
INC      := -Iinclude -Ihost

all: build/test_host build/libhello.so build/libbad_abi.so build/libno_exports.so

build:
	mkdir -p build

build/libhello.so: plugins/hello_plugin.cpp include/blockverse_plugin.h | build
	$(CXX) $(CXXFLAGS) $(INC) -fPIC -fvisibility=hidden -shared $< -o $@

build/libbad_abi.so: plugins/bad_abi_plugin.cpp include/blockverse_plugin.h | build
	$(CXX) $(CXXFLAGS) $(INC) -fPIC -fvisibility=hidden -shared $< -o $@

build/libno_exports.so: plugins/no_exports_plugin.cpp | build
	$(CXX) $(CXXFLAGS) $(INC) -fPIC -shared $< -o $@

build/test_host: tests/test_host.cpp host/plugin_host.cpp host/plugin_host.hpp include/blockverse_plugin.h | build
	$(CXX) $(CXXFLAGS) $(INC) tests/test_host.cpp host/plugin_host.cpp -ldl -o $@

test: all
	./build/test_host ./build/libhello.so ./build/libbad_abi.so ./build/libno_exports.so

clean:
	rm -rf build

.PHONY: all test clean
