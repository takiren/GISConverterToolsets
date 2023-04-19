#include "GmlDoc.h"
namespace gistool {
rx::xml_node<>* GmlDoc::find_node(rx::xml_node<>* node, std::string& name) {
  if (node == nullptr) return nullptr;
  std::queue<rx::xml_node<>*> que;
  que.push(node);
  while (!que.empty()) {
    auto front = que.front();
    que.pop();

    for (auto it = front->first_node(); it; it = it->next_sibling()) {
      if (it->name_size() > 0) {
        std::cout << "Queued: " << it->name() << std::endl;
        if (std::string(it->name()) == name) return it;
      }

      que.push(it);
    }
  }
}

double GmlDoc::sizex() { return 0; }

double GmlDoc::sizey() { return 0; }

int GmlDoc::cell_size_x() { return 0; }

int GmlDoc::cell_size_y() { return 0; }

double GmlDoc::tlx() { return 0.0; }

double GmlDoc::tly() { return 0.0; }

double GmlDoc::brx() { return 0.0; }

double GmlDoc::bry() { return 0.0; }

void GmlDoc::cellsize_internal(int* nx, int* ny) {}

std::vector<double> GmlDoc::size_lat_lon() {
  auto envnode = this->find_node_by_name(std::string("gml:Envelope"));
  std::vector<double> ret(4);

  std::stringstream ss_lowercorner{envnode->first_node("gml:low")->value()};
  for (size_t i = 0; i < 2; i++) {
    std::string buff("");
    std::getline(ss_lowercorner, buff, ' ');
    ret[i] = std::stod(buff);
  }

  std::stringstream ss_uppercorner{envnode->first_node("gml:high")->value()};
  for (size_t i = 0; i < 2; i++) {
    std::string buff("");
    std::getline(ss_uppercorner, buff, ' ');
    ret[i + 2] = std::stod(buff);
  }
  return ret;
}

std::vector<int> GmlDoc::size_cells() {
  auto envnode = this->find_node_by_name(std::string("gml:GridEnvelope"));
  std::vector<int> ret(2);

  std::stringstream ss_uppercorner{envnode->first_node("gml:high")->value()};
  for (size_t i = 0; i < 2; i++) {
    std::string buff("");
    std::getline(ss_uppercorner, buff, ' ');
    ret[2] = std::stoi(buff);
  }
  return std::vector<int>();
}

GmlDoc::GmlDoc(std::string filename)
    : document(new rx::xml_document<>()),
      file(new rx::file<>(filename.c_str())) {}

bool GmlDoc::try_parse() {
  try {
    this->document->parse<0>(this->file->data());
    return true;
  } catch (rx::parse_error& err) {
    return false;
  }
}
}  // namespace gistool
