# 1. 设置CMake最低版本和项目信息
cmake_minimum_required(VERSION 3.10)
project(DetectorProject CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(PkgConfig REQUIRED)

pkg_search_module(NCNN REQUIRED ncnn)
message(STATUS "NCNN library paths: ${NCNN_LIBRARIES}")
message(STATUS "NCNN include paths: ${NCNN_INCLUDE_DIRS}")
message(STATUS "NCNN ncnn libraries dir: ${NCNN_LIBRARY_DIRS}")

pkg_search_module(ZXING REQUIRED zxing)
message(STATUS "Found ZXing-cpp includes: ${ZXING_INCLUDE_DIRS}")
message(STATUS "Found ZXing-cpp libraries: ${ZXING_LIBRARIES}")
message(STATUS "Found ZXing-cpp libraries dir: ${ZXING_LIBRARY_DIRS}")

add_library(detect STATIC detect.cpp)

target_include_directories(detect PRIVATE
    ${NCNN_INCLUDE_DIRS}
    ${ZXING_INCLUDE_DIRS}
)

target_link_directories(detect PRIVATE
    ${NCNN_LIBRARY_DIRS}
    ${ZXING_LIBRARY_DIRS}
)

target_link_libraries(detect PRIVATE
    ${NCNN_LIBRARIES}
    ${ZXING_LIBRARIES}
)

find_package(Threads REQUIRED)
target_link_libraries(detect PRIVATE Threads::Threads)

# (可选) 打印输出可执行文件的位置
message(STATUS "Executable is available at: ${CMAKE_BINARY_DIR}/detect")

install(TARGETS detect
    ARCHIVE DESTINATION lib
)