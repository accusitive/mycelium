#define BOOST_LOG_DYN_LINK 1

#include "iostream"
#include <boost/asio.hpp>
#include <boost/asio/io_context.hpp>
#include <boost/bind/bind.hpp>
#include <boost/log/trivial.hpp>
#include <boost/system/detail/error_code.hpp>
#include <netinet/in.h>
#include <sys/socket.h>
#include <thread>

using boost::asio::ip::tcp;

using std::endl;
class MinecraftSession {
public:
  MinecraftSession(boost::asio::io_context &io_context) : socket_(io_context) {}
  tcp::socket &socket() { return socket_; }
  void start() {
    socket_.async_read_some(
        boost::asio::buffer(data_, max_length),
        boost::bind(&MinecraftSession::handle_read, this,
                    boost::asio::placeholders::error,
                    boost::asio::placeholders::bytes_transferred));
  }
  void handle_read(const boost::system::error_code &error,
                   size_t bytes_transferred) {
    BOOST_LOG_TRIVIAL(info) << "READ " << bytes_transferred << " BYTES";
    int length = read_var_int();
    int id = read_var_int();
    BOOST_LOG_TRIVIAL(info) << "len: " << length << " id: " << id;

    // Handshake
    int pv = read_var_int();

    int string_length = read_var_int();
    char address[255];
    for (int i = 0; i < string_length; i++) {
      address[i] = read_byte();
    }
    BOOST_LOG_TRIVIAL(info)
        << "PV " << pv << " ADD " << string_length << " !!" << address;
  }

private:
  int read_integer() { return ((int *)data_)[(this->location += 4) - 4]; }
  char read_byte() { return data_[++this->location]; }
  float read_float() { return data_[this->location++]; }
  int read_var_int() {
    int value = 0;
    int length = 0;
    char currentByte;
    while (true) {
      currentByte = read_byte();
      printf("%x\n", currentByte);
      value |= (currentByte & 0x7F) << (length * 7);
      length += 1;
      if (length > 5) {
        BOOST_LOG_TRIVIAL(fatal) << "Varint too large. Exiting.";
        exit(1);
      }
      if ((value & 0x80) != 0x80) {
        break;
      }
    }
    return value;
  }
  tcp::socket socket_;
  enum { max_length = 1024 };
  char data_[max_length];
  int location = 0;
};
class MinecraftServer {

public:
  MinecraftServer(int port, boost::asio::io_context &io_context)
      : io_context_(io_context), port(port), running(true),
        acceptor_(io_context, tcp::endpoint(tcp::v4(), port)) {}

  void start() {
    this->start_accept();
    BOOST_LOG_TRIVIAL(info) << "Server started on port " << this->port;

    io_context_.run();
  }

private:
  void start_accept() {
    MinecraftSession *new_session = new MinecraftSession(io_context_);
    acceptor_.async_accept(new_session->socket(),
                           boost::bind(&MinecraftServer::handle_accept, this,
                                       new_session,
                                       boost::asio::placeholders::error));
  }
  void handle_accept(MinecraftSession *new_session,
                     const boost::system::error_code &error) {
    BOOST_LOG_TRIVIAL(info) << "Accepted client.";
    new_session->start();
    start_accept();
  }
  int port;
  bool running;
  boost::asio::io_context &io_context_;
  tcp::acceptor acceptor_;
};
int main() {
  boost::asio::io_context io_context;
  MinecraftServer server(25575, io_context);
  server.start();

  return 0;
}
