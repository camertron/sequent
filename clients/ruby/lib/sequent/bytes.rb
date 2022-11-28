# frozen_string_literal: true

module Sequent
  class Bytes
    def initialize(data)
      @data = data
      @pos = 0
    end

    def get_slice(len)
      @data[@pos...(@pos + len)].tap { @pos += len }
    end

    def get_u8(count = 1)
      get('C', count, 1)
    end

    def get_u64(count = 1)
      get('Q>', count, 8)
    end

    def get_i64(count = 1)
      get('q>', count, 8)
    end

    def get_f64(count = 1)
      get('G', count, 8)
    end

    private

    def get(fmt, count, unit_bytesize)
      format_str = count == 1 ? fmt : "#{fmt}#{count}"

      data = if count == 1
        @data.unpack1(format_str, offset: @pos)
      else
        @data.unpack(format_str, offset: @pos)
      end

      @pos += unit_bytesize * count

      data
    end
  end
end
