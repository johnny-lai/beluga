
module Beluga
  module Commands
    class Base
      attr_accessor :app, :image
      
      def initialize(app, options = {})
        @app = app
        @image = options["image"]
      end
    end
  end
end
