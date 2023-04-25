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
        if (std::string(it->name()) == name) return it;
      }

      que.push(it);
    }
  }
  return nullptr;
}

double GmlDoc::sizex() { return 0; }

double GmlDoc::sizey() { return 0; }

int GmlDoc::cell_size_x() { return 0; }

int GmlDoc::cell_size_y() { return 0; }

double GmlDoc::tlx() { return 0.0; }

double GmlDoc::tly() { return 0.0; }

double GmlDoc::brx() { return 0.0; }

double GmlDoc::bry() { return 0.0; }

bool GmlDoc::write_gtiff(fs::path outpath) {
  if (!this->try_parse()) return false;
  using namespace std;
  double transform[6] = {0};
  this->get_transform(transform);
  auto node = this->find_node_by_name(string("gml:tupleList"));
  if (!node) return false;
  auto cells = this->size_cells();

  if (!fs::exists(outpath)) {
    fs::create_directory(outpath);
  }
  fs::path out_path;

  out_path.append(file_path.filename().c_str());
  out_path.replace_extension(".tiff");
  cout << out_path.string() << endl;
  outpath.append(out_path.string());
  this->dataset = gdriver->Create(outpath.string().c_str(), cells[0], cells[1],
                                  1, GDT_Float32, NULL);

  std::stringstream ss{node->value()};
  std::string buf;
  std::getline(ss, buf);
  auto val = new float[cells[0]];

  for (size_t row = 0; row < cells[1]; row++) {
    for (size_t col = 0; col < cells[0]; col++) {
      if (std::getline(ss, buf)) {
        std::stringstream splited(buf);
        std::string h_buf("");
        for (size_t i = 0; i < 2; i++) {
          getline(splited, h_buf, ',');
          if (i == 1) val[col] = stof(h_buf);
        }
      } else {
        val[col] = -9999.f;
      }
    }

    dataset->GetRasterBand(1)->RasterIO(GF_Write, 0, row, cells[0], 1, val,
                                        cells[0], 1, GDT_Float32, 0, 0);
  }

  dataset->SetGeoTransform(transform);
  dataset->GetRasterBand(1)->SetNoDataValue(-9999);
  dataset->SetSpatialRef(spatialref);
  GDALClose(dataset);
  delete[] val;

  return true;
}

void GmlDoc::cellsize_internal(int* nx, int* ny) {}

std::vector<double> GmlDoc::size_lat_lon() {
  using namespace std;
  auto envnode = this->find_node_by_name(std::string("gml:Envelope"));
  std::vector<double> ret(4);
  /// �֋X�㉺���Ə���Ɛ������Ă��邪���͈Ⴄ�B
  /// �n�\�ɋ�`��`�����Ƃ��A����̌o�x�A�ܓx�����ꂼ��
  /// ret[1]�Aret[2]�ɓ���Ă���B
  /// �E����ret[3]�Aret[0]�ɓ���Ă���B
  ///
  /// ret[0] �ܓx�̉���
  /// ret[1] �o�x�̉���
  /// ret[2] �ܓx�̏��
  /// ret[3] �o�x�̏��
  std::stringstream ss_lowercorner{
      envnode->first_node("gml:lowerCorner")->value()};
  std::string b;
  // getline(ss_lowercorner, b);
  for (size_t i = 0; i < 2; i++) {
    std::string buff("");
    std::getline(ss_lowercorner, buff, ' ');
    cout << buff << endl;
    ret[i] = std::stod(buff);
  }

  std::stringstream ss_uppercorner{
      envnode->first_node("gml:upperCorner")->value()};
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
    ret[i] = std::stoi(buff) + 1;
  }
  return ret;
}

GmlDoc::GmlDoc(fs::path filename)
    : document(new rx::xml_document<>()),
      file(new rx::file<>(filename.string().c_str())),
      dataset(nullptr),
      gdriver(nullptr),
      file_path(filename),
      spatialref(nullptr) {}

GmlDoc::~GmlDoc() {
  delete document;
  delete file;
}

bool GmlDoc::try_parse() {
  try {
    this->document->parse<0>(this->file->data());
    return true;
  } catch (rx::parse_error& err) {
    return false;
  }
}
ConverterManager::ConverterManager(int epsg = 6668)
    : spatialref(OGRSpatialReference()) {
  this->spatialref.importFromEPSG(epsg);
}

void ConverterManager::add_queue(fs::path path) {
  // doque.emplace(path.string());
}

}  // namespace gistool
