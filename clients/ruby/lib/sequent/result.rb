# frozen_string_literal: true

module Sequent
  class Result
    attr_reader :row_count, :column_count, :column_names, :rows

    def initialize(row_count, column_count, column_names, rows)
      @row_count = row_count
      @column_count = column_count
      @column_names = column_names
      @rows = rows
    end
  end
end
