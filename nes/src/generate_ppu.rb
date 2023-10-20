#!/usr/bin/ruby

def output( code = '' )
    $output ||= ""
    $indentation = 0 if $indentation == nil
    lines = code.strip.split( "\n" ).map do |line|
        balanced_braces = line.count("{") > 0 && line.count("{") == line.count("}")

        if balanced_braces == false && (line.strip.end_with?( '}' ) || line.strip.end_with?( '];' ) || line.strip.end_with?( '},' ))
            $indentation -= 4
        end
        line = (' ' * $indentation) + line.strip
        if line.end_with?( '{' ) || line.end_with?( '[' )
            $indentation += 4
        end

        line
    end

    $output << lines.join( "\n" )
    $output << "\n"
end

def write_output_to_file( filename )
    STDERR.puts "Generating #{filename}..."
    File.open( get_path( filename ), "wb" ) do |fp|
        fp.puts $output.gsub ";;", ";"
    end
end

def get_path( filename )
    File.join( File.dirname( __FILE__ ), filename )
end

def read_lines( filename )
    File.read( filename )
        .strip
        .split( "\n" )
        .map( &:strip )
        .each_with_index
        .map
        .select { |line, _| !line.start_with?( '#' ) && !line.empty? }
        .map { |line, index| [line, index + 1] }
end

class PatternValue < Struct.new( :value )
    def matches( value )
        self.value == value
    end
end

class PatternRange < Struct.new( :first, :last )
    def matches( value )
        value >= self.first && value <= self.last
    end
end

class PatternCycle < Struct.new( :first, :step, :last )
    def matches( value )
        value >= self.first && value <= self.last && ((value - self.first) % (self.step - self.first)) == 0
    end
end

class Rule < Struct.new( :scanlines, :dots, :code )
    def matches_scanline( scanline )
        self.scanlines.find { |pattern| pattern.matches( scanline ) }
    end

    def matches_dot( dot )
        self.dots.find { |pattern| pattern.matches( dot ) }
    end
end

def translate( symbols, chunk )
    chunk.split( ',' ).map do |value|
        value = value.strip
        symbols.each do |src, dst|
            value.gsub!( src, dst )
        end

        if value =~ /\A(\d+?)\.\.(\d+?)\.\.(\d+?)\Z/
            PatternCycle.new $1.to_i, $2.to_i, $3.to_i
        elsif value =~ /\A(\d+?)\.\.(\d+?)\Z/
            PatternRange.new $1.to_i, $2.to_i
        elsif value =~ /\A(\d+)\Z/
            PatternValue.new $1.to_i
        else
            throw "Unparsable value: `#{value}`"
        end
    end
end

def read_data( filename )
    rules, last_chunks = [], []
    symbols = {}
    read_lines( filename ).map do |line, line_number|
        if line =~ /%define\s+(\S+)\s+(\S+)/
            symbols[ $1 ] = $2
            symbols = symbols.sort { |a, b| b[0].length <=> a[0].length }.to_h
            next
        end

        chunks = line.split( /\s{2,}/ )
        if chunks.length == 1
            chunks = [last_chunks[0], last_chunks[1], chunks[0]]
        end
        last_chunks = chunks

        throw "Messed up spacing between columns on line #{line_number}!" if chunks.length != 3

        scanlines, dots, code = translate( symbols, chunks[0] ), translate( symbols, chunks[1] ), chunks[2]
        rule = Rule.new( scanlines, dots, code )
        previous_rule = rules.find { |previous| previous.scanlines == rule.scanlines && previous.dots == rule.dots }
        if previous_rule != nil
            previous_rule.code += "\n"
            previous_rule.code += rule.code
        else
            rules << rule
        end
    end

    rules
end

def append_action( array, actions )
    if array[-1] != nil && array[-1][:actions] == actions
        array[-1][:times] += 1
    else
        array << { times: 1, actions: actions }
    end
end

def extract_actions_per_scanline
    actions_per_scanline = []
    (0..261).each do |scanline|
        rules_matching_this_scanline = RULES.select { |rule| rule.matches_scanline( scanline ) }
        actions_for_this_scanline = []
        (0..340).each do |dot|
            code = rules_matching_this_scanline.select { |rule| rule.matches_dot( dot ) }.map { |rule| rule.code }.join( "\n" )
            actions_for_this_scanline << code
        end
        append_action( actions_per_scanline, actions_for_this_scanline )
        # puts actions_per_scanline[0][:actions].each_with_index.to_a.map(&:inspect)
        # exit 1
    end

    actions_per_scanline
end

def build_chunks_per_scanline( actions_per_scanline )
    actions_per_scanline.map do |entry|
        actions = entry[:actions]

        chunks = []
        index = 0
        while index < actions.length
            rule = actions[index]
            length = nil

            1.upto( RULE_REDUCE_LIMIT ) do |offset|
                if actions[index + offset] == actions[index]
                    length = offset
                    break
                end
            end

            if length == nil
                if !chunks.empty? && chunks[-1][:times] == 1
                    chunks[-1][:actions] << rule
                else
                    chunks << { times: 1, actions: [rule] }
                end
                index += 1
            else
                current_index = index
                pattern = actions[index...(index + length)]
                times = 0

                while actions[current_index...(current_index + length)] == pattern
                    current_index += length
                    times += 1
                end

                chunks << { times: times, actions: pattern }
                index += times * length
            end

        end

        { times: entry[:times], chunks: chunks }
    end
end

def extract_unique_actions( chunks_per_scanline )
    unique_actions = {}
    chunks_per_scanline.each do |scanline_entry|
        scanline_entry[:chunks].each do |chunk_entry|
            chunk_entry[:actions].each do |action|
                next if unique_actions.has_key? action
                unique_actions[ action ] = unique_actions.length
            end
        end
    end

    unique_actions
end

def print_debug_info( chunks_per_scanline )
    chunks_per_scanline.each do |scanline_entry|
        puts "#{scanline_entry[:times]}x"
        index = 0
        scanline_entry[:chunks].each do |chunk_entry|
            length = chunk_entry[:actions].length * chunk_entry[:times]
            puts "    #{chunk_entry[:times]}x (#{index}..#{index + length - 1})"
            chunk_entry[:actions].each do |action|
                puts "        #{action.gsub( "\n", " ; " )}"
            end
            index += length
        end
    end
end

def generate_header

    output <<-EOS
        // WARNING: This file was autogenerated; DO NOT EDIT.

        #![macro_use]
        macro_rules! ppu_scheduling_logic {
            () => {

                struct Scanline {
                    times: u16,
                    first_chunk_index: u8,
                    last_chunk_index: u8
                }

                struct Chunk {
                    times: u16,
                    first_action_index: u16,
                    last_action_index: u16
                }
    EOS

    output

end

def generate_footer

    output "}"
    output "}"

end

def generate_scanlines( constant_name, chunks_per_scanline )

    output "static #{constant_name}: &'static [Scanline] = #{constant_name}_CONST;"
    output "const #{constant_name}_CONST: &'static [Scanline] = &["

    index = 0
    chunks_per_scanline.each do |scanline|
        length = scanline[:chunks].length
        output <<-EOS
            Scanline {
                times: #{scanline[:times]},
                first_chunk_index: #{index},
                last_chunk_index: #{index + length - 1}
            },
        EOS

        index += length
    end

    output "];"
    output

end

def generate_chunks( constant_name, chunks_per_scanline )

    output "static #{constant_name}: &'static [Chunk] = #{constant_name}_CONST;"
    output "const #{constant_name}_CONST: &'static [Chunk] = &["

    index = 0
    chunks_per_scanline.flat_map { |scanline| scanline[:chunks] }.each do |chunk|
        length = chunk[:actions].length
        output <<-EOS
            Chunk {
                times: #{chunk[:times]},
                first_action_index: #{index},
                last_action_index: #{index + length - 1}
            },
        EOS

        index += length
    end
    output "];"
    output

end

def generate_actions( unique_actions )

    unique_actions.each do |action, index|
        callback_name = "action_%02i" % [ index ]
        output <<-EOS
            #[allow(unused_variables)]
            fn #{callback_name}< T: Private >( ppu: &mut T ) {
                #{action};
            }
        EOS
        output
    end

end

def generate_action_table_builder( chunks_per_scanline, unique_actions )

    output <<-EOS
        unsafe fn get_action< T: Private >( index: usize ) -> fn( &mut T ) {
    EOS

    index = 0
    output "match index {"
    chunks_per_scanline.each_with_index do |scanline, scanline_index|
        scanline[:chunks].each_with_index do |chunk, chunk_index|
            chunk[:actions].each_with_index do |action, dot_index|
                callback_name = "action_%02i" % [ unique_actions[action] ]
                output <<-EOS
                    #{index} => #{callback_name}::< T > as fn( &mut T ),
                EOS
                index += 1
            end
        end
    end
    output "_ => fast_unreachable!()"
    output "}"
    output "}"

end

RULE_REDUCE_LIMIT = 8
DATA_FILENAME = get_path( 'rp2c02_scheduling.dat' )
RULES = read_data( DATA_FILENAME )
DEBUG_MODE = ARGV.include? "--debug"

STDERR.puts "Optimizing RP2C02 scheduling..."
chunks_per_scanline = build_chunks_per_scanline( extract_actions_per_scanline() )
unique_actions = extract_unique_actions( chunks_per_scanline )
print_debug_info( chunks_per_scanline ) if DEBUG_MODE

STDERR.puts "#{chunks_per_scanline.length} scanlines; #{unique_actions.length} unique actions"

generate_header()
generate_scanlines( "SCANLINES", chunks_per_scanline )
generate_chunks( "CHUNKS", chunks_per_scanline )
generate_actions( unique_actions )
generate_action_table_builder( chunks_per_scanline, unique_actions )
generate_footer()

write_output_to_file "rp2c02_scheduler.rs"
