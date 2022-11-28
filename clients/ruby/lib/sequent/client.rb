# frozen_string_literal: true

require 'ffi-rzmq'

module Sequent
  class Client
    class Error < StandardError; end
    class MalformedResponseError < Error; end

    DEFAULT_HOST = '127.0.0.1'
    DEFAULT_PORT = 9087

    attr_reader :host

    def initialize(host = DEFAULT_HOST, port = DEFAULT_PORT)
      @host = host
      @port = port

      @context = ZMQ::Context.new
      @socket = @context.socket(ZMQ::REQ)
      @socket.connect("tcp://#{@host}:#{@port}")
    end

    def exec_query(query)
      @socket.send_string(query)

      header_bytes, row_bytes = [].tap do |messages|
        @socket.recv_strings(messages)
      end

      row_count, column_count, column_names = deserialize_header(header_bytes)
      rows = deserialize_rows(row_bytes, row_count, column_count)

      Result.new(row_count, column_count, column_names, rows)
    end

    private

    def deserialize_header(header_bytes)
      msg = Bytes.new(header_bytes)

      if msg.get_slice(4) != 'SQNT'
        raise MalformedResponseError, 'malformed response from server'
      end

      row_count, column_count = msg.get_u64(2)

      column_names = Array.new(column_count) do
        len = msg.get_u64
        msg.get_slice(len).force_encoding(Encoding::UTF_8)
      end

      [row_count, column_count, column_names]
    end

    def deserialize_rows(row_bytes, row_count, column_count)
      msg = Bytes.new(row_bytes)

      Array.new(row_count) do
        Array.new(column_count) do
          case msg.get_u8
            when 0
              nil
            when 1
              msg.get_i64
            when 2
              msg.get_f64
            when 3
              len = msg.get_u64
              msg.get_slice(len).force_encoding(Encoding::UTF_8)
            when 4
              len = msg.get_u64
              msg.get_slice(len)
            else
              raise MalformedResponseError, 'malformed response from server'
          end
        end
      end
    end
  end
end
