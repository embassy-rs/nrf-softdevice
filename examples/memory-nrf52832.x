MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* NRF52832 with Softdevice S132 7.x and 6.x */
  FLASH : ORIGIN = 0x00026000, LENGTH = 512K - 152K
  RAM : ORIGIN = 0x20007af8, LENGTH = 64K - 31480
}
