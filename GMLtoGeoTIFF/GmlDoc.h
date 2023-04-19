#ifndef GML_DOC_H
#define GML_DOC_H
#include <gdal_priv.h>
#include <geotiff.h>
#include <geotiffio.h>

#include <filesystem>
#include <iostream>
#include <queue>
#include <sstream>
#include <string>
#include <vector>

#include "rapidxml.hpp"
#include "rapidxml_utils.hpp"

namespace gistool {
namespace rx = rapidxml;
namespace fs = std::filesystem;

namespace helper {
template <typename T>
std::vector<T> get_val(std::stringstream stream, char delim) {
  std::vector<T> ret(2);
  std::string buff;
  std::getline(stream, buff, ' ');
}
}  // namespace helper

class GmlDoc {
 private:
  rx::xml_document<>* document;
  rx::file<>* file;
  void cellsize_internal(int* nx, int* ny);

 public:
  GmlDoc() = delete;
  explicit GmlDoc(std::string filename);
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
};
}  // namespace gistool

#endif  // !GML_DOC_H
