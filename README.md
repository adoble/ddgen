The tool takes in a TOML specification of a SPI device  and generates a low level 
driver (the driver "peripheral access crate" _PAC_).




The tool assumes the following architecture for a full-blown driver:

```
    ┌──────┐        ┌────────┐       ┌────────┐
    │ User │  Calls │ Driver │ Calls │ Driver │
    │ App  ├───────►│  HAL   ├──────►│  PAC   │
    └──────┘        └────────┘       └─┬────┬─┘
                                       │    │
                           Communicates│    │
                               using   │    │
                                SPI    │    │
                                       │    │
                                       │    │
                                   ┌───▼────┴───┐
                                   │   Device   │
                                   └────────────┘
```

The generated driver PAC (Peripheral Access Crate) is used to access the commands of the device and provides 
an API that can be used by a device HAL (Hardware Access Layer) that provides another API for the user that can:

- Abstract away the complexity of directly using the registers
- Provide a different domain model that stresses certain features of the device. 

The generated PAC provides a serializiation, deserialization model to a rust struct that removes the explict handling 
of bits, fields etc. from the SPI data.  

# Example

Using the following, minimal, TOML specification for the MCP23017 multiplexer (`mcp23017.toml`):

```toml
version= "0.0.1"

[device]
name      = "MCP23017"
wprd_size = 8


# Register addressed assume that IOCON.BANK = 0 which is its reset value

[commands.GPIOA]
opcode = 0x09
description = """The GPIO register reflects the value on the port.
Reading from this register reads the port. Writing to this
register modifies the Output Latch (OLAT) register"""

[commands.GPIOA.request]
gpio0 = { bits = "0", enum = "logic_level" }
gpio1 = { bits = "1", enum = "logic_level" }
gpio2 = { bits = "2", enum = "logic_level" }
gpio3 = { bits = "3", enum = "logic_level" }
gpio4 = { bits = "4", enum = "logic_level" }
gpio5 = { bits = "5", enum = "logic_level" }
gpio6 = { bits = "6", enum = "logic_level" }
gpio7 = { bits = "7", enum = "logic_level" }

[enum.logic_level]
low  = 0
high = 1


[commands.IODIRA]
opcode     = 0x00
description = "I/O direction register. Controls the direction of the data I/O"

[commands.IODIRA.request]
iodir0 = { bits = "0", enum = "direction" }
iodir1 = { bits = "1", enum = "direction" }
iodir2 = { bits = "2", enum = "direction" }
iodir3 = { bits = "3", enum = "direction" }
iodir4 = { bits = "4", enum = "direction" }
iodir5 = { bits = "5", enum = "direction" }
iodir6 = { bits = "6", enum = "direction" }
iodir7 = { bits = "7", enum = "direction" }

[enum.direction]
output = 0
input  = 1
```

And running the tool with:

```sh
    ddgen   ./definitions/mcp23017.toml ./generated
```

generates the driver project with source code in the provided directory.

# Using the API 

TODO

# Limitations

- Currently only handles 8 bit SPI words.