#include <gdal_priv.h>
#include <geotiff.h>
#include <geotiffio.h>
#include <stdio.h>

#include <filesystem>
#include <fstream>
#include <iostream>
#include <queue>
#include <sstream>
#include <string>
#include <vector>

#include "GmlDoc.h"
#include "cxxopts.hpp"
#include "rapidxml.hpp"
#include "rapidxml_utils.hpp"
using String = std::string;

namespace fs = std::filesystem;
namespace rx = rapidxml;

using namespace std;
using namespace gistool;
int main(int argc, char* argv) {
  setlocale(LC_CTYPE, "");
  cxxopts::Options options("opts");
  GDALAllRegister();
  CPLPushErrorHandler(CPLQuietErrorHandler);

  try {
    std::string folder;
    options.add_options()("folder", "Folder name",
                          cxxopts::value<std::string>(folder));
  } catch (cxxopts::OptionException& e) {
    std::cout << options.usage() << std::endl;
  }

  String file("FG-GML-5338-62-00-DEM5A-20210115.xml");
  String s("gml:tupleList");
  auto doc = new GmlDoc(file);
  doc->try_parse();

  auto cellsizenode = doc->find_node_by_name(String("gml:GridEnvelope"));

  double transformer[6] = {};
  transformer[0] = 138.25;
  transformer[1] = (-138.25 + 138.2625) / 225;
  transformer[2] = 0;
  transformer[3] = 35.841666667;
  transformer[4] = 0;
  transformer[5] = (+35.833333333 - 35.841666667) / 150;
  auto node = doc->find_node_by_name(s);

  {
    GDALDriver* gdriver = nullptr;
    gdriver = GetGDALDriverManager()->GetDriverByName("GTiff");

    int grid_x_size = 225;
    int grid_y_size = 150;

    auto dataset = gdriver->Create("tes2.tiff", grid_x_size, grid_y_size, 1,
                                   GDT_Float32, NULL);
    OGRSpatialReference sref;
    sref.importFromEPSG(6668);

    std::stringstream ss{node->value()};
    std::string buf;
    getline(ss, buf);

    // for (size_t i = 0; i < 5; i++) {
    //   getline(ss, buf);
    //   //cout << "sdff" << buf << endl;
    //   stringstream ss2{buf};
    //   //cout << ss2.str() << endl;
    //   String b("");
    //   int cntt = 0;
    //   while (getline(ss2, b, ',')) {
    //     if (cntt == 1) cout << b << endl;
    //     cntt++;
    //   }
    // }
    // return 0;

    float var[225] = {0};

    for (size_t row = 0; row < grid_y_size; row++) {
      for (size_t col = 0; col < grid_x_size; col++) {
        std::getline(ss, buf);

        std::stringstream splited(buf);
        std::string h_buf("");
        int i = 0;
        while (getline(splited, h_buf, ',')) {
          if (i == 1) var[col] = stof(h_buf);
          i++;
        }
      }
      dataset->GetRasterBand(1)->RasterIO(GF_Write, 0, row, grid_x_size, 1, var,
                                          grid_x_size, 1, GDT_Float32, 0, 0);
    }

    dataset->SetGeoTransform(transformer);
    dataset->SetSpatialRef(&sref);
    GDALClose(dataset);
    GDALDestroyDriverManager();
  }
  return 0;
}

/*

String extension(".tif"), tiffname("test");
tiffname.append(extension);

double transform[6];

GDALDriver* gtiff_driver;
gtiff_driver = GetGDALDriverManager()->GetDriverByName("GTiff");
auto dataset = gtiff_driver->Create(tiffname.c_str(), 100, 100, 1,
GDT_Float64, NULL); float* row_buff = (float*)CPLMalloc(sizeof(float) *
100);

for (size_t i = 0; i < 100; i++)
{
        for (size_t j = 0; j < 100; j++)
        {
                row_buff[j] = i * 10 + j;
        }
        dataset->GetRasterBand(1)->RasterIO(GF_Write, 0, i, 100, 1,
row_buff, 100, 1, GDT_Float32, 0, 0);
}
GDALClose(dataset);*/
