MEMORY
{
  /* On-chip Flash memory (last 8K page reserved for storage) */
  FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 504K

  /* On-chip SRAM (SRAM1+SRAM2+SRAM3 combined) */
  RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 256K
}
