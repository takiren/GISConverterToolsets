cmake_minimum_required(VERSION 3.22)

project(tlgml VERSION 0.1 LANGUAGES CXX)

set(CMAKE_TOOLCHAIN_FILE "C:/vcpkg/scripts/buildsystems/vcpkg.cmake")
include(C:/vcpkg/scripts/buildsystems/vcpkg.cmake)
find_package(GDAL CONFIG REQUIRED)
find_package(ZLIB REQUIRED)
add_definitions(-DUNICODE)
add_definitions(-D_UNICODE)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
file(GLOB HEADERS RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" "*.h")
file(GLOB SOURCES RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" "*.cpp")
list(APPEND CGLOB
    ${CMAKE_CURRENT_SOURCE_DIR}/cppglob/src/glob.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/cppglob/src/fnmatch.cpp
)

add_executable(tlgml ${SOURCES} ${HEADERS} ${CGLOB})
target_include_directories(tlgml PRIVATE cppglob/include)
target_link_libraries(tlgml PRIVATE GDAL::GDAL)