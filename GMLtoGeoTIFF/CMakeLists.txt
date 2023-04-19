cmake_minimum_required(VERSION 3.22)

project(GMLtoGTIFF VERSION 0.1 LANGUAGES CXX)

set(CMAKE_TOOLCHAIN_FILE "C:/vcpkg/scripts/buildsystems/vcpkg.cmake")
include(C:/vcpkg/scripts/buildsystems/vcpkg.cmake)
find_package(GDAL CONFIG REQUIRED)

add_definitions(-DUNICODE)
add_definitions(-D_UNICODE)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
file(GLOB HEADERS RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" "*.h")
file(GLOB SOURCES RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" "*.cpp")

add_executable(GMLtoGTIFF ${SOURCES} ${HEADERS})
target_link_libraries(GMLtoGTIFF PRIVATE GDAL::GDAL)