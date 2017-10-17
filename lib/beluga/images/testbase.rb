
module Beluga
  module Images
    class Testbase < Base
      def build_root
        "#{Beluga.root}/docker/testbase"
      end
      
      def environment
        "RAILS_ROOT=#{app.root} APP_DOCKER_LABEL=#{image} DEVBASE_LABEL=#{app.images["devbase"].image} DIGEST=#{app.digest} "
      end
    end
  end
end
