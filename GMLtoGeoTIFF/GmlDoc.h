#ifndef GML_DOC_H
#define GML_DOC_H
#include <gdal_priv.h>
#include <geotiff.h>
#include <geotiffio.h>
#include <ogr_spatialref.h>

#include <filesystem>
#include <iostream>
#include <queue>
#include <sstream>
#include <string>
#include <vector>

#include "rapidxml.hpp"
#include "rapidxml_utils.hpp"

namespace gistool {
#ifdef _DEBUG

#define DEBUG_LOG(TEXT) std::cout << #TEXT << std::endl;
#else
#define DEBUG_LOG(TEXT)
#endif  // DEBUG

namespace rx = rapidxml;
namespace fs = std::filesystem;
class GmlDoc;

namespace helper {}  // namespace helper

class GmlDoc {
 private:
  rx::xml_document<>* document;
  rx::file<>* file;
  void cellsize_internal(int* nx, int* ny);
  GDALDataset* dataset;
  GDALDriver* gdriver;
  OGRSpatialReference* spatialref;

  fs::path file_path;

 public:
  GmlDoc() = delete;
  explicit GmlDoc(fs::path filename);

  virtual ~GmlDoc();
  bool try_parse();
  rx::xml_node<>* find_node(rx::xml_node<>* node, std::string& name);
  rx::xml_node<>* find_node_by_name(std::string& name) {
    return this->find_node(document->first_node(), name);
  }

  std::vector<double> size_lat_lon();
  std::vector<int> size_cells();
  double sizex();
  double sizey();

  int cell_size_x();
  int cell_size_y();
  double tlx();
  double tly();
  double brx();
  double bry();

  bool write_gtiff();

  inline void get_transform(double transform[6]) {
    auto env_latlon = this->size_lat_lon();
    auto env_grid = this->size_cells();
    transform[0] = env_latlon[1];
    transform[1] = (env_latlon[3] - env_latlon[1]) / env_grid[0];
    transform[2] = 0;
    transform[3] = env_latlon[2];
    transform[4] = 0;
    transform[5] = (env_latlon[0] - env_latlon[2]) / env_grid[1];
  }

  inline void set_transform(double transform[6]) {
    dataset->SetGeoTransform(transform);
  }

  inline void set_spatialref(OGRSpatialReference& sref) { spatialref = &sref; }

  inline void set_gdaldriver(GDALDriver* driver) { this->gdriver = driver; }
};

class ConverterManager {
 private:
  OGRSpatialReference spatialref;
  std::queue<std::unique_ptr<GmlDoc>>
      doque;  /// documentÇ∆queueÇÇ©ÇØÇΩÇ®óVÇ—ÅB
 public:
  explicit ConverterManager(int epsg);
  void add_queue(fs::path path);
};

}  // namespace gistool

#endif  // !GML_DOC_H
