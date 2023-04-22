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
  {
    concurrent::ThreadPoolExecutor executor;
    cout << "Thread count: " << executor.thread_count() << endl;

    cxxopts::Options options("opts");
    GDALAllRegister();
    CPLPushErrorHandler(CPLQuietErrorHandler);
    bool blist = false;
    bool combine = false;
    try {
      options.add_options()("l,list", "list source files",
                            cxxopts::value<bool>()->default_value("false"))(
          "c,combine", "Combine geotiffs",
          cxxopts::value<bool>()->default_value("false"));
      auto result = options.parse(argc, argv);
      blist = result["list"].as<bool>();
      combine = result["combine"].as<bool>();

    } catch (cxxopts::OptionException& e) {
      std::cout << options.usage() << std::endl;
      std::cout << "Invalid args" << std::endl;
      return -1;
    }

    vector<fs::path> sources;

    for (const auto& it :
         fs::recursive_directory_iterator(fs::current_path().append("gmls"))) {
      if (!it.is_directory()) {
        if (it.path().extension() == ".xml") {
          sources.push_back(it.path());
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

    for (const auto& it : sources) {
      GmlDoc* gdoc = new GmlDoc(it);
      gdoc->set_gdaldriver(gdriver);
      gdoc->set_spatialref(sref);
      gdoc->write_gtiff();
      /*auto ftr = executor.submit([&] {
        gdoc->write_gtiff();
        delete gdoc;
      });*/

      delete gdoc;
    }
  }
  GDALDestroyDriverManager();
  return 0;
}
