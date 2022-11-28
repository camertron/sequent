$:.unshift File.join(File.dirname(__FILE__), 'lib')
require 'sequent/version'

Gem::Specification.new do |s|
  s.name     = 'sequent'
  s.version  = ::Sequent::VERSION
  s.authors  = ['Cameron Dutro']
  s.email    = ['camertron@gmail.com']
  s.homepage = 'http://github.com/camertron/sequent-ruby'

  s.description = s.summary = 'A client for talking to a Sequent server.'
  s.platform = Gem::Platform::RUBY

  s.add_dependency 'ffi-rzmq'

  s.executables << 'sequent'

  s.require_path = 'lib'
  s.files = Dir['{lib,spec}/**/*', 'Gemfile', 'CHANGELOG.md', 'README.md', 'Rakefile', 'sequent.gemspec']
end
