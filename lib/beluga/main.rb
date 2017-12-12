require 'ostruct'
require 'json'

module Beluga
  class Main
    attr_accessor :options
    
    def initialize(argv)
      @argv = argv
      @options = parse_options(@argv)
    end
    
    def app
      @app ||= Beluga::RailsApp.new(options.app)
    end
    
    def run
      command = @argv.shift
      case command
      when "command"
        while subcommand = @argv.shift do
          case subcommand
          when "list"
            app.commands.each do |k, v|
              puts({ k => v.options }.to_json)
            end
          when "info"
            puts app.commands[@argv.shift].options.to_json
          else
            raise "Unknown command: command #{subcommand}"
          end
        end
      when "image"
        while subcommand = @argv.shift do
          case subcommand
          when "list"
            app.images.each do |k, v|
              puts({ k => v.options }.to_json)
            end
          when "info"
            puts get_image.options.to_json
          when "build", "push", "pull", "clean"
            get_image.send(subcommand)
          when "label"
            puts get_image.image
          else
            raise "Unknown command: image #{subcommand}"
          end
        end
      when "digest"
        puts app.digest
      else
        cmd = app.commands[command]
        image = @options.image || app.images[cmd.image]
        image.run(cmd, @argv)
      end
    end
  
    # - Internal -----------------------------------------------------------------
    def parse_options(argv)
      argv << '-h' if argv.empty?
      options = OpenStruct.new(image: nil, app: '.')
      OptionParser.new do |opts|
        opts.banner = <<~eos
          Usage: beluga [options] <commands>
                 beluga [options] digest
                 beluga [options] command <command-commands>
                 beluga [options] image <image-commands>
                 
                 beluga [options] digest
                   Prints the digest of the rails application
                  
                 beluga [options] command <command-commands>
                   * list
                     List all commands
                   * info <cmd>
                     Prints info on <cmd> command
                   
                 beluga [options] image <image-commands>
                   * list
                     List all image
                   * info [<img>|<IMAGE>]
                     Prints info on <img> image
                   * label [<img>|<IMAGE>]
                     Prints the docker label of specifed image
                   * build [<img>|<IMAGE>]
                     Builds specified docker image
                   * push [<img>|<IMAGE>]
                     Pushes specified docker image
                   * pull [<img>|<IMAGE>]
                     Pulls specified docker image
                   * clean [<img>|<IMAGE>]
                     Cleans working data for building specifed docker image
                      
        eos

        opts.on("-aAPP", "--app=APP", "Location of Application. Defaults to '.'") do |v|
          options.app = v
        end
        opts.on("-iIMAGE", "--image=IMAGE", "Name of image. Defaults to devbase.") do |v|
          options.image = app.images[v]
        end
        opts.on_tail("-h", "--help", "Show this message") do
          puts opts
          exit
        end
      end.parse!
      options
    end

    def get_image
      return options.image if options.image
      return app.images[@argv.shift] if !@argv.empty?
      app.images["devbase"]
    end
  end
end
