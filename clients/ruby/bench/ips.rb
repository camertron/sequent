# frozen_string_literal: true

require 'sequent-client'
require 'benchmark/ips'

module Sequent
  class ServerProcess
    EXE = 'target/debug/sequent-server'

    attr_reader :host, :port, :log_file

    def initialize(host = SequentClient::DEFAULT_HOST, port = SequentClient::DEFAULT_PORT)
      @host = host
      @port = port
    end

    def start
      @log_file = File.expand_path('server.out', __dir__)

      Dir.chdir(File.expand_path(File.join('..', '..', '..'), __dir__)) do
        unless File.exist?(EXE)
          puts "Could not find #{EXE}, attempting to build"
          system 'cargo build --bin sequent-server'

          unless File.exist?(EXE)
            raise "Could not find #{EXE} after attempting to build"
          end
        end

        @pid = Process.spawn("#{EXE} -f test.sqlite -b #{@host}:#{@port} > #{log_file}", pgroup: true)
      end

      sleep 1

      # waitpid returns the pid if the process has exited
      if Process.waitpid(@pid, Process::WNOHANG)
        raise 'Server did not start up successfully, aborting benchmark'
      end
    end

    def stop
      raise 'Server is not running' unless @pid

      # kill process group containing our child server process
      pgid = Process.getpgid(@pid)
      Process.kill('HUP', -pgid)
      Process.detach(pgid)
    end
  end
end

server_process = Sequent::ServerProcess.new('127.0.0.1', 9087)
STDOUT.write 'Starting server...'
server_process.start
puts ' done'
puts "Logging server output to #{server_process.log_file}"

client = Sequent::Client.new('127.0.0.1', 9087)

Benchmark.ips do |x|
  x.report do
    client.exec_query('select * from users')
  end
end

STDOUT.write 'Stopping server...'
server_process.stop

puts " done"
