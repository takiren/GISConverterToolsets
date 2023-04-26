#include <gdal.h>
#include <gdal_alg.h>
#include <gdal_priv.h>
#include <gdal_utils.h>
#include <gdal_vrt.h>
#include <geotiff.h>
#include <geotiffio.h>
#include <ogr_spatialref.h>
#include <stdio.h>
#include <vrtdataset.h>

#include <filesystem>
#include <fstream>
#include <iostream>
#include <queue>
#include <sstream>
#include <string>
#include <thread>

#include "GmlDoc.h"
#include "cxxopts.hpp"
#include "rapidxml.hpp"
#include "rapidxml_utils.hpp"
#include "threadpool.h"

using String = std::string;

namespace fs = std::filesystem;
namespace rx = rapidxml;

using namespace std;
using namespace gistool;

int main(int argc, char* argv[]) {
  GDALAllRegister();
  CPLPushErrorHandler(CPLQuietErrorHandler);
  cxxopts::Options options("opts");
  bool blist = false;
  bool combine = false;
  bool recursive = false;
  fs::path source_directory("gmls");
  fs::path target_directory("out");
  try {
    options.add_options()("l,list", "list source files",
                          cxxopts::value<bool>()->default_value("false"))(
        "c,combine", "Combine geotiffs",
        cxxopts::value<bool>()->default_value("false"))(
        "s,source", "Souce directory",
        cxxopts::value<std::string>()->default_value("gmls"))(
        "r,recursive", "Search recursively",
        cxxopts::value<bool>()->default_value("false"))(
        "o,output", "Target directory",
        cxxopts::value<std::string>()->default_value("out"));

    auto result = options.parse(argc, argv);
    blist = result["list"].as<bool>();
    combine = result["combine"].as<bool>();
    recursive = result["recursive"].as<bool>();
    source_directory.assign(result["source"].as<std::string>());
    target_directory.assign(result["output"].as<std::string>());
  } catch (cxxopts::OptionException& e) {
    std::cout << options.usage() << std::endl;
    std::cout << "Invalid args" << std::endl;
    return -1;
  }

  {
    vector<fs::path> sources;

    if (recursive) {
      for (const auto& it :
           fs::recursive_directory_iterator(source_directory)) {
        if (!it.is_directory()) {
          if (it.path().extension() == ".xml") {
            sources.push_back(it.path());
          }
        }
      }
    } else {
      for (const auto& it : fs::directory_iterator(source_directory)) {
        if (!it.is_directory()) {
          if (it.path().extension() == ".xml") {
            sources.push_back(it.path());
          }
        }
      }
    }

    if (blist) {
      for (auto& e : sources) {
        cout << e.filename() << endl;
      }
      return 0;
    }

    GDALDriver* gdriver = nullptr;
    gdriver = GetGDALDriverManager()->GetDriverByName("GTiff");
    OGRSpatialReference sref;
    sref.importFromEPSG(6668);
    {
      concurrent::ThreadPoolExecutor executor;
      cout << "Thread count: " << executor.thread_count() << endl;
      for (const auto& it : sources) {
        GmlDoc* gdoc = new GmlDoc(it);
        gdoc->set_gdaldriver(GetGDALDriverManager()->GetDriverByName("GTiff"));
        gdoc->set_spatialref(sref);

        auto ftr = executor.submit([gdoc, &target_directory] {
          gdoc->write_gtiff(target_directory);
          delete gdoc;
        });
      }
    }
  }

  /// GeotiffをVRTへ。
  if (!combine) {
    GDALDestroyDriverManager();
    return 0;
  }
  auto vrt_driver = GDALDriverManager().GetDriverByName("VRT");
  if (!vrt_driver) {
    cout << "Not found VRT driver" << endl;
    GDALDestroyDriverManager();
    return -1;
  }

  GDALDestroyDriverManager();
  return 0;
}
