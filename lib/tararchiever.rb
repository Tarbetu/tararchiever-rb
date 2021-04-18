require "tararchiever/version"
require "rutie"

module TarArchiever
  Rutie.new(:tar_archiever).init "initialize_rust_lib", __dir__
end
