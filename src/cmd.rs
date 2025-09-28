/// Enumeration of instructions for the ST7789V2 display.
pub enum Commands {
    Nop = 0x00, // No Operation
    // Description: This command does nothing and does not affect the display state.
    // Use: Typically used as a placeholder to ensure a clock cycle completes with no effect.
    SwReset = 0x01, // Software Reset
    // Description: Resets all control registers to their default values without affecting frame memory.
    // Use: It’s essential to wait 5ms after issuing this command before sending new commands.
    RddId = 0x04, // Read Display Identification Information
    // Description: Returns a 24-bit display ID that includes manufacturer, module, and version information.
    // Use: Used for identifying the specific hardware connected to the interface.
    RddSt = 0x09, // Read Display Status
    // Description: Reads the current status of the display, such as inversion, sleep, and display modes.
    // Use: Used to check the active state of the display, including inversion and partial modes.
    SlpIn = 0x10, // Enter Sleep Mode (SLPIN)
    // Description: Puts the display into sleep mode, reducing power consumption but retaining memory contents.
    // Use: Requires 5ms delay before issuing new commands after sleep is activated.
    SlpOut = 0x11, // Exit Sleep Mode (SLPOUT)
    // Description: Exits the sleep mode and powers up the display. Restarts the display's scanning and power circuits.
    // Use: Requires 120ms delay before issuing further commands after waking up.
    PtlOn = 0x12, // Enter Partial Mode (PTLON)
    // Description: Activates partial mode where only a portion of the display is refreshed.
    // Use: Useful for applications requiring power savings by limiting the active area of the screen.
    NorOn = 0x13, // Enter Normal Mode (NORON)
    // Description: Returns the display to full-screen, normal mode operation, cancelling partial mode.
    // Use: When switching back from partial mode to full-screen operation.
    InvOff = 0x20, // Display Inversion Off (INVOFF)
    // Description: Turns off display inversion where pixel colors are inverted (negative colors).
    // Use: Used to restore normal color display.
    InvOn = 0x21, // Display Inversion On (INVON)
    // Description: Activates display inversion, reversing pixel colors.
    // Use: Can be used for creating visual effects or enhancing contrast.
    GamSet = 0x26, // Gamma Set (GAMSET)
    // Description: Configures the display gamma curve for different color contrasts.
    // Use: Used to adjust display contrast depending on the required visual quality.
    DispOff = 0x28, // Display Off (DISPOFF)
    // Description: Turns off the display, while keeping internal memory and settings intact.
    // Use: Saves power when the display is not needed but the system is still active.
    DispOn = 0x29, // Display On (DISPON)
    // Description: Turns on the display and resumes outputting data from memory.
    // Use: Used to reactivate the display after a period of inactivity.
    CaSet = 0x2A, // Column Address Set (CASET)
    // Description: Defines the start and end addresses for columns in a memory write operation.
    // Use: Necessary for defining a rectangular area of the display for drawing.
    RaSet = 0x2B, // Row Address Set (RASET)
    // Description: Defines the start and end addresses for rows in a memory write operation.
    // Use: Used in conjunction with CASET for defining the display area.
    RamWr = 0x2C, // Memory Write (RAMWR)
    // Description: Writes pixel data to memory using the addresses defined by CASET and RASET.
    // Use: Commonly used to send image or graphical data to the display.
    RamRd = 0x2E, // Memory Read (RAMRD)
    // Description: Reads pixel data from memory within the area defined by CASET and RASET.
    // Use: Useful for retrieving image data stored in display memory.
    PtlAr = 0x30, // Partial Area (PTLAR)
    // Description: Defines the active region for partial mode, limiting the active rows for refreshing.
    // Use: Specifies the area of the display to be refreshed in partial mode.
    VScrDef = 0x33, // Vertical Scrolling Definition (VSCRDEF)
    // Description: Defines the top and bottom margins for vertical scrolling.
    // Use: Required when implementing vertical scrolling on the display.
    TEOFF = 0x34, // Tearing Effect Line OFF (TEOFF)
    // Description: Disables the tearing effect signal, which synchronizes display refresh with the frame rate.
    // Use: Used when synchronizing with external video sources is unnecessary.
    TEON = 0x35, // Tearing Effect Line ON (TEON)
    // Description: Enables the tearing effect signal for frame synchronization.
    // Use: Useful for avoiding visual tearing when the display's refresh is out of sync with data input.
    MadCtl = 0x36, // Memory Access Control (MADCTL)
    /// 6 bits: MY, MX, MV, ML, BGR, MH.
    /// - MY (Bit 7): Row address order (0 = top-to-bottom, 1 = bottom-to-top)
    /// - MX (Bit 6): Column address order (0 = left-to-right, 1 = right-to-left)
    /// - MV (Bit 5): Row/column exchange (0 = normal, 1 = reverse)
    /// - ML (Bit 4): Vertical refresh order (0 = top-to-bottom, 1 = bottom-to-top)
    /// - BGR (Bit 3): RGB/BGR order (0 = RGB, 1 = BGR)
    /// - MH (Bit 2): Horizontal refresh order (0 = left-to-right, 1 = right-to-left)
    // Description: Controls the orientation of the display (rotation, mirroring) and color order (RGB/BGR).
    // Use: Configures the display's orientation and pixel arrangement for different viewing angles.
    ColMod = 0x3A, // Pixel Format Set (COLMOD)
    /// 3 bits: D6, D5, D4.
    /// - D6: RGB interface color format (101 = 65K colors, 110 = 262K colors)
    /// - D5-D4: Control interface color format (011 = 12-bit, 101 = 16-bit, 110 = 18-bit)
    // Description: Sets the color format (e.g., 12-bit, 16-bit, or 18-bit).
    // Use: Essential for defining the bit depth of each pixel during data transfers.
    WrMemC = 0x3C, // Write Memory Continue (WRMEMC)
    // No bit-level details, continues writing to the display memory.
    // Description: Continuously writes data to memory from the current location.
    // Use: Useful for fast, large data writes without resetting the address pointers.
    RdMemC = 0x3E, // Read Memory Continue (RDMEMC)
    // No bit-level details, continues reading from the display memory.
    // Description: Continuously reads data from memory starting from the current address.
    // Use: Used when reading large amounts of display data without resetting the address pointers.
    Ste = 0x44, // Set Tear Scanline (STE)
    // No bit-level details, sets a specific scanline for tear synchronization.
    // Description: Sets a specific scanline at which the tearing effect occurs.
    // Use: Helps to synchronize the display refresh to a specific scanline.
    GScan = 0x45, // Get Scanline (GSCAN)
    // No bit-level details, returns the current scanline being drawn.
    // Description: Returns the current scanline position of the display driver.
    // Use: Useful for determining the current position in the vertical scanning cycle.
    WrDisBV = 0x51, // Write Display Brightness (WRDISBV)
    /// 8 bits: DBV[7:0].
    /// - DBV[7:0]: Display brightness value (00 = lowest brightness, FF = highest brightness)
    // Description: Sets the brightness of the display by controlling the backlight intensity.
    // Use: Adjusts display brightness for different lighting conditions.
    RdDisBV = 0x52, // Read Display Brightness (RDDISBV)
    /// 8 bits: DBV[7:0].
    /// - DBV[7:0]: Current display brightness value
    // Description: Reads the current brightness setting of the display.
    // Use: Useful for checking the current brightness level.
    WrCtrLD = 0x53, // Write CTRL Display (WRCTRLD)
    /// 3 bits: BCTRL, DD, BL.
    /// - BCTRL: Brightness control block (0 = Off, 1 = On)
    /// - DD: Display dimming (0 = Off, 1 = On)
    /// - BL: Backlight control (0 = Off, 1 = On)
    // Description: Controls display brightness and CABC (Content Adaptive Brightness Control).
    // Use: Configures dynamic brightness control for power saving and visual optimization.
    RdCtrLD = 0x54, // Read CTRL Display (RDCTRLD)
    /// 3 bits: BCTRL, DD, BL.
    /// - BCTRL: Brightness control block (0 = Off, 1 = On)
    /// - DD: Display dimming (0 = Off, 1 = On)
    /// - BL: Backlight control (0 = Off, 1 = On)
    // Description: Reads the current control display settings, such as brightness control.
    // Use: Allows checking the configuration of the control display settings.
    WrCACE = 0x55, // Write CABC (WRCACE)
    /// 4 bits: CECTRL, C1, C0.
    /// - CECTRL: Color enhancement control (0 = Off, 1 = On)
    /// - C1-C0: Color enhancement level (00 = Low, 01 = Medium, 11 = High)
    // Description: Configures the Content Adaptive Brightness Control (CABC) settings.
    // Use: Used to enable or adjust the CABC feature, which adapts brightness to content.
    RdCABC = 0x56, // Read CABC (RDCABC)
    // No bit-level details, reads the current CABC setting.
    // Description: Reads the current CABC setting.
    // Use: Useful for determining the current CABC mode.
    WrCABCMB = 0x5E, // Write CABC Minimum Brightness (WRCABCMB)
    // No bit-level details, sets the minimum brightness value for CABC.
    // Description: Sets the minimum brightness level for CABC operation.
    // Use: Ensures that the brightness never falls below a certain threshold even when CABC is active.
    RdCABCMB = 0x5F, // Read CABC Minimum Brightness (RDCABCMB)
    // No bit-level details, reads the minimum brightness value for CABC.
    // Description: Reads the current minimum brightness setting for CABC.
    // Use: Checks the current lower limit for display brightness under CABC.
    RdABCSDR = 0x68, // Read Automatic Brightness Control Self-Diagnostic Result (RDABCSDR)
    // No bit-level details, reads diagnostic results from the brightness control system.
    // Description: Returns diagnostic results for automatic brightness control functionality.
    // Use: Useful for debugging and verifying the health of the brightness control circuitry.
    RdId1 = 0xDA, // Read ID1 (RDID1)
    // No bit-level details, reads the first 8 bits of the display identification.
    // Description: Reads the first part of the display identification (Manufacturer ID).
    // Use: Used for identifying the manufacturer of the display module.
    RdId2 = 0xDB, // Read ID2 (RDID2)
    // No bit-level details, reads the second 8 bits of the display identification.
    // Description: Reads the second part of the display identification (Module/Driver ID).
    // Use: Useful for identifying the specific module version.
    RdId3 = 0xDC, // Read ID3 (RDID3)
                  // No bit-level details, reads the third 8 bits of the display identification.
                  // Description: Reads the third part of the display identification (Additional ID data).
                  // Use: Provides additional identification details about the display hardware.
}
