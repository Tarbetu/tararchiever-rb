# Tararchiever

This is my first Ruby gem which powered by Rust code. It was an good pratice but I haven't tested it.

I have tried to do use Rust compression libraries for Rust code. It's working but not useful for this time.  

WIP.

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'tararchiever'
```

And then execute:

    $ bundle install

Or install it yourself as:

    $ gem install tararchiever

## Usage

```ruby 
Archiever.compress_dir :path
```

Compresses the path. 

```ruby
Archiever.decompress_tar :path
```

Decompresses the path.

It would works with tar.gz. There is some options for LZ4, XZ and ZSTD but I'm too lazy to write it. 

## Development

After checking out the repo, run `bin/setup` to install dependencies. Then, run `rake test` to run the tests. You can also run `bin/console` for an interactive prompt that will allow you to experiment.

To install this gem onto your local machine, run `bundle exec rake install`. To release a new version, update the version number in `version.rb`, and then run `bundle exec rake release`, which will create a git tag for the version, push git commits and tags, and push the `.gem` file to [rubygems.org](https://rubygems.org).

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/[USERNAME]/tararchiever. This project is intended to be a safe, welcoming space for collaboration, and contributors are expected to adhere to the [code of conduct](https://github.com/tarbetu/tararchiever/blob/master/CODE_OF_CONDUCT.md).


## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

## Code of Conduct

Everyone interacting in the Tararchiever project's codebases, issue trackers, chat rooms and mailing lists is expected to follow the [code of conduct](https://github.com/tarbetu/tararchiever/blob/master/CODE_OF_CONDUCT.md).
