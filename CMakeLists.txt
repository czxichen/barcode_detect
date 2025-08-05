cmake_minimum_required(VERSION 3.10)
project(detect CXX)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(NCNN_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/third_party/ncnn"
    CACHE PATH "Path to the NCNN source directory")

if(NOT EXISTS "${NCNN_SOURCE_DIR}/CMakeLists.txt")
    message(FATAL_ERROR "NCNN source directory not found or is invalid at path: ${NCNN_SOURCE_DIR}\n"
        "Please specify the correct path using: -DNCNN_SOURCE_DIR=/path/to/ncnn")
endif()

message(STATUS "Using NCNN source from: ${NCNN_SOURCE_DIR}")

# 设置 NCNN 的编译选项 (这些保持不变)
set(NCNN_OPENMP OFF CACHE BOOL "Disable Build NCNN openmp" FORCE)
set(NCNN_VULKAN OFF CACHE BOOL "Disable Build NCNN vulkan" FORCE)
set(NCNN_SIMPLEOCV 1 CACHE BOOL "Enable Build NCNN simple_cv" FORCE)
set(NCNN_BUILD_TOOLS OFF CACHE BOOL "Disable Build NCNN tools" FORCE)
set(NCNN_BUILD_EXAMPLES OFF CACHE BOOL "Disable Build NCNN examples" FORCE)
set(NCNN_BUILD_BENCHMARK OFF CACHE BOOL "Disable Build NCNN benchmark" FORCE)

add_subdirectory(${NCNN_SOURCE_DIR} ncnn-build)

add_library(detect STATIC
    detect.cpp
)

target_link_libraries(detect PUBLIC ncnn)

if(NOT CMAKE_INSTALL_PREFIX)
    set(CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/dist" CACHE PATH "Installation directory")
endif()

message(STATUS "Installation directory set to: ${CMAKE_INSTALL_PREFIX}")

install(TARGETS detect
    ARCHIVE DESTINATION lib
)

install(TARGETS ncnn
    EXPORT ncnn-targets
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
    INCLUDES DESTINATION include
)