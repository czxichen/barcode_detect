cmake_minimum_required(VERSION 3.10)
project(detect CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

macro(force_release_build target_name)
    string(REPLACE " " ";" RELEASE_FLAGS_LIST "${CMAKE_CXX_FLAGS_RELEASE}")
    message(STATUS "Forcing Release options for target: ${target_name}")
    target_compile_options(${target_name} PRIVATE
        $<$<NOT:$<CONFIG:Release>>:${RELEASE_FLAGS_LIST}>
    )
    target_compile_definitions(${target_name} PRIVATE
        $<$<NOT:$<CONFIG:Release>>:NDEBUG>
    )
endmacro()

# --- NCNN 依赖设置 ---
set(NCNN_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/third_party/ncnn"
    CACHE PATH "Path to the NCNN source directory")

if(NOT EXISTS "${NCNN_SOURCE_DIR}/CMakeLists.txt")
    message(FATAL_ERROR "NCNN source directory not found or is invalid at path: ${NCNN_SOURCE_DIR}\n"
        "Please specify the correct path using: -DNCNN_SOURCE_DIR=/path/to/ncnn")
endif()

message(STATUS "Using NCNN source from: ${NCNN_SOURCE_DIR}")

# 设置 NCNN 的编译选项
set(NCNN_OPENMP OFF CACHE BOOL "Disable Build NCNN openmp" FORCE)
set(NCNN_VULKAN OFF CACHE BOOL "Disable Build NCNN vulkan" FORCE)
set(NCNN_SIMPLEOCV ON CACHE BOOL "Enable Build NCNN simple_cv" FORCE)
set(NCNN_BUILD_TOOLS OFF CACHE BOOL "Disable Build NCNN tools" FORCE)
set(NCNN_BUILD_EXAMPLES OFF CACHE BOOL "Disable Build NCNN examples" FORCE)
set(NCNN_BUILD_BENCHMARK OFF CACHE BOOL "Disable Build NCNN benchmark" FORCE)

add_subdirectory(${NCNN_SOURCE_DIR} ncnn-build)

# --- ZXING-CPP 依赖设置 ---
set(ZXING_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/third_party/zxing-cpp"
    CACHE PATH "Path to the ZXING-CPP source directory")

if(NOT EXISTS "${ZXING_SOURCE_DIR}/CMakeLists.txt")
    message(FATAL_ERROR "ZXING-CPP source directory not found or is invalid at path: ${ZXING_SOURCE_DIR}\n"
        "Please specify the correct path using: -DZXING_SOURCE_DIR=/path/to/zxing-cpp")
endif()

message(STATUS "Using ZXING-CPP source from: ${ZXING_SOURCE_DIR}")

set(ZXING_READERS ON CACHE BOOL "Build zxing-cpp barcode readers" FORCE)
set(ZXING_WRITERS OFF CACHE BOOL "Disable Build zxing-cpp barcode writers" FORCE)
set(ZXING_EXAMPLES OFF CACHE BOOL "xx" FORCE)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "Disable Build zxing-cpp examples" FORCE)

add_subdirectory(${ZXING_SOURCE_DIR} zxing-cpp-build)

add_library(detect STATIC
    detect.cpp
)

force_release_build(ncnn)
force_release_build(ZXing)

target_link_libraries(detect PUBLIC ncnn ZXing)

if(NOT CMAKE_INSTALL_PREFIX)
    set(CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/install" CACHE PATH "Installation directory")
endif()

message(STATUS "Installation directory set to: ${CMAKE_INSTALL_PREFIX}")

install(TARGETS detect
    ARCHIVE DESTINATION lib
)