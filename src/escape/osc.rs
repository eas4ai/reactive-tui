/// Operating System Command (OSC) sequences
#[derive(Debug, Clone, PartialEq)]
pub enum OSCAction {
    /// Set window title
    SetTitle(String),

    /// Set icon name
    SetIconName(String),

    /// Set window title and icon name
    SetTitleAndIcon(String),

    /// Set color palette entry
    SetColor {
        /// Color palette index (0-255 typically)
        index: u16,
        /// RGB hex string like "#RRGGBB"
        color: String,
    },

    /// Reset color palette entry
    ResetColor(u16),

    /// Hyperlink
    Hyperlink {
        /// Optional parameters like id=xxx
        params: Option<String>,
        /// The URI to link to
        uri: String,
    },

    /// Clipboard operations
    Clipboard {
        /// Which clipboard to use
        clipboard: ClipboardType,
        /// Operation to perform
        operation: ClipboardOperation,
        /// Data for set operations
        data: Option<String>,
    },

    /// Notification
    Notification {
        /// Optional notification title
        title: Option<String>,
        /// Notification body text
        body: String,
    },

    /// Set current working directory
    CurrentDirectory(String),

    /// Set current file
    CurrentFile(String),

    /// iTerm2 inline images protocol
    InlineImage {
        /// Optional image name
        name: Option<String>,
        /// Optional size (width, height)
        size: Option<(u32, u32)>,
        /// Whether to preserve aspect ratio
        preserve_aspect: bool,
        /// Whether to display inline
        inline: bool,
        /// Base64 decoded image data
        data: Vec<u8>,
    },

    /// Kitty graphics protocol
    KittyGraphics {
        /// Graphics action to perform
        action: KittyGraphicsAction,
        /// Image format
        format: Option<KittyImageFormat>,
        /// Transmission method
        transmission: Option<KittyTransmission>,
        /// Image ID
        id: Option<u32>,
        /// Placement ID
        placement_id: Option<u32>,
        /// Image data
        data: Option<Vec<u8>>,
    },

    /// Query terminal capabilities
    QueryCapability(String),

    /// Unknown or unhandled OSC
    Unknown {
        /// OSC number
        number: u16,
        /// OSC data
        data: String,
    },
}

/// Type of clipboard to use
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardType {
    /// Primary selection (X11)
    Primary,
    /// System clipboard
    Clipboard,
    /// Text selection
    Selection,
}

/// Clipboard operation to perform
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardOperation {
    /// Copy to clipboard
    Copy,
    /// Paste from clipboard
    Paste,
    /// Clear clipboard
    Clear,
}

/// Kitty graphics protocol action
#[derive(Debug, Clone, PartialEq)]
pub enum KittyGraphicsAction {
    /// Transmit image data
    Transmit,
    /// Transmit and display image
    TransmitAndDisplay,
    /// Query image
    Query,
    /// Place image at cursor
    PlaceCursor,
    /// Delete image
    Delete,
    /// Delete all images
    DeleteAll,
}

/// Kitty image format
#[derive(Debug, Clone, PartialEq)]
pub enum KittyImageFormat {
    /// RGB format
    RGB,
    /// RGBA format with alpha
    RGBA,
    /// PNG format
    PNG,
}

/// Kitty image transmission method
#[derive(Debug, Clone, PartialEq)]
pub enum KittyTransmission {
    /// Direct transmission
    Direct,
    /// File path
    File,
    /// Temporary file
    TempFile,
    /// Shared memory
    SharedMemory,
}

impl OSCAction {
    /// Parse an OSC sequence from its components
    pub fn parse(params: &[u8]) -> Option<Self> {
        // Find the first semicolon to split number from data
        let semicolon = params.iter().position(|&b| b == b';');

        if let Some(pos) = semicolon {
            let number_bytes = &params[..pos];
            let data_bytes = &params[pos + 1..];

            // Parse the OSC number
            let number = String::from_utf8_lossy(number_bytes).parse::<u16>().ok()?;

            let data = String::from_utf8_lossy(data_bytes).to_string();

            match number {
                0 => Some(OSCAction::SetTitleAndIcon(data)),
                1 => Some(OSCAction::SetIconName(data)),
                2 => Some(OSCAction::SetTitle(data)),

                4 => {
                    // Color palette change
                    if let Some(semi) = data.find(';') {
                        let index = data[..semi].parse().ok()?;
                        let color = data[semi + 1..].to_string();
                        Some(OSCAction::SetColor { index, color })
                    } else {
                        None
                    }
                }

                104 => {
                    // Reset color palette
                    let index = data.parse().ok()?;
                    Some(OSCAction::ResetColor(index))
                }

                7 => {
                    // Current working directory
                    Some(OSCAction::CurrentDirectory(data))
                }

                8 => {
                    // Hyperlink
                    parse_hyperlink(&data)
                }

                9 => {
                    // Notification (ConEmu/iTerm2 style)
                    if let Some(semi) = data.find(';') {
                        let title = data[..semi].to_string();
                        let body = data[semi + 1..].to_string();
                        Some(OSCAction::Notification {
                            title: if title.is_empty() { None } else { Some(title) },
                            body,
                        })
                    } else {
                        Some(OSCAction::Notification {
                            title: None,
                            body: data,
                        })
                    }
                }

                52 => {
                    // Clipboard operations
                    parse_clipboard(&data)
                }

                1337 => {
                    // iTerm2 extensions
                    parse_iterm2(&data)
                }

                _ => Some(OSCAction::Unknown { number, data }),
            }
        } else {
            // No semicolon, might be a query or simple command
            let data = String::from_utf8_lossy(params).to_string();
            data.strip_prefix('?')
                .map(|stripped| OSCAction::QueryCapability(stripped.to_string()))
        }
    }
}

fn parse_hyperlink(data: &str) -> Option<OSCAction> {
    // Format: params;URI
    // params can be empty, or contain id=xxx
    if let Some(semi) = data.find(';') {
        let params = &data[..semi];
        let uri = data[semi + 1..].to_string();

        Some(OSCAction::Hyperlink {
            params: if params.is_empty() {
                None
            } else {
                Some(params.to_string())
            },
            uri,
        })
    } else if data.is_empty() {
        // Empty hyperlink clears the current link
        Some(OSCAction::Hyperlink {
            params: None,
            uri: String::new(),
        })
    } else {
        None
    }
}

fn parse_clipboard(data: &str) -> Option<OSCAction> {
    // Format: clipboard;operation;data
    let parts: Vec<&str> = data.splitn(3, ';').collect();

    if parts.len() < 2 {
        return None;
    }

    let clipboard = match parts[0] {
        "p" => ClipboardType::Primary,
        "c" | "s" => ClipboardType::Clipboard,
        "q" => ClipboardType::Selection,
        _ => return None,
    };

    let (operation, data) = if parts[1] == "?" {
        // Query/paste operation
        (ClipboardOperation::Paste, None)
    } else if parts.len() == 3 {
        // Copy operation with data
        (ClipboardOperation::Copy, Some(parts[2].to_string()))
    } else if parts[1].is_empty() {
        // Clear operation
        (ClipboardOperation::Clear, None)
    } else {
        // Copy operation with data in second position
        (ClipboardOperation::Copy, Some(parts[1].to_string()))
    };

    Some(OSCAction::Clipboard {
        clipboard,
        operation,
        data,
    })
}

fn parse_iterm2(data: &str) -> Option<OSCAction> {
    // iTerm2 uses various subcommands after 1337
    if let Some(stripped) = data.strip_prefix("File=") {
        // Inline image protocol
        parse_iterm2_image(stripped)
    } else {
        data.strip_prefix("CurrentDir=")
            .map(|stripped| OSCAction::CurrentDirectory(stripped.to_string()))
    }
}

fn parse_iterm2_image(data: &str) -> Option<OSCAction> {
    // Format: name=xxx;size=WxH;inline=1;preserveAspectRatio=1:base64data
    let colon = data.find(':')?;
    let params = &data[..colon];
    let image_data = &data[colon + 1..];

    let mut name = None;
    let mut size = None;
    let mut inline = true;
    let mut preserve_aspect = true;

    for param in params.split(';') {
        if let Some(eq) = param.find('=') {
            let key = &param[..eq];
            let value = &param[eq + 1..];

            match key {
                "name" => name = Some(value.to_string()),
                "size" => {
                    if let Some(x) = value.find('x') {
                        if let (Ok(w), Ok(h)) =
                            (value[..x].parse::<u32>(), value[x + 1..].parse::<u32>())
                        {
                            size = Some((w, h));
                        }
                    }
                }
                "inline" => inline = value == "1",
                "preserveAspectRatio" => preserve_aspect = value == "1",
                _ => {}
            }
        }
    }

    // Decode base64 image data
    let data = base64_decode(image_data)?;

    Some(OSCAction::InlineImage {
        name,
        size,
        preserve_aspect,
        inline,
        data,
    })
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    // Simple base64 decoder (in production, use a proper base64 crate)
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::new();
    let mut bits = 0u32;
    let mut bit_count = 0;

    for byte in input.bytes() {
        if byte == b'=' {
            break;
        }

        let value = TABLE.iter().position(|&b| b == byte)? as u32;
        bits = (bits << 6) | value;
        bit_count += 6;

        if bit_count >= 8 {
            bit_count -= 8;
            output.push((bits >> bit_count) as u8);
            bits &= (1 << bit_count) - 1;
        }
    }

    Some(output)
}
