# Language Processor Unit (LPU)

Let's start out with an idea _because I'm bored_: **What if a processor had an ALU (Arithmetic Logic Unit) that was an LLM?**

Basically, we shifted from having a processor that is exact and deterministic to one that is probabilistic and generative. This new paradigm called "Soft Computing" allows us to work with data that is unstructured, messy, or subjective in a way that traditional computing struggles with. In short, we can handle ambiguity and fuzzy logic, which is becoming more common in real‑world applications.

This project explores the idea of implementing a simple processor that has memory (RAM), a control unit, registers, and an ALU that is powered by a language model. The instruction set is designed to allow us to write code that can interact with the language model in a structured way, while still allowing for the flexibility and creativity of natural language prompts.

Think of this assembly language as a middle ground between traditional programming languages and natural language prompts, where we can write code that is more structured and modular than natural language prompts, but still allows us to work with multi-modal data in a way that is more intuitive and flexible than traditional programming languages.

## Why?

I really wanted to imagine a future where we can write code where we don't have to worry about edge cases or complex logic to handle unstructured data. Instead, we can just write code that describes what we want to achieve, and let the language model handle the complexity of how to achieve it. In short, **we can write code that is more focused on the "what" rather than the "how"**.

Here's an example of what a program written in this assembly language looks like:

```
; Program: Room Comfort Adjustment System
; Objective: Adjust the room's temperature and lighting based on sensor data to achieve optimal physical comfort.
; Output: Adjusted temperature and lighting settings.

; Load sensor data and user feedback.
LF   X1, "examples/data/room_sensor_data.json"
LS   X2, "It's too dark to read and I am sweating."

PSH  X1                     ; Push the sensor data the context stack for processing.

; Sense: Brief description of the current state of the room based on sensor data.
LS  X3, "A sentence that describes the current state of the room with only the following information: temperature in celsius, and light intensity in percentage."
MAP  X4, X3

; Build the context for adjustments.
DRP                         ; Clear context stack to build a new context for adjustments.

LS   X3, "Current room state:"
PSH  X3
PSH  X4                     ; Push the summarised state for context.

SNP  X31                    ; Save the current room state context for later adjustment.

; Build the temperature feedback context for adjustments.
CLR                         ; Clear context stack to classify user temperature feedback for adjustments.
PSH  X2                     ; Push the user feedback to the context stack for processing.

LS   X3, "Classify the following temperature feedback into a category (TOO_COLD, TOO_WARM, COMFORTABLE, UNRELATED) and intensity level (Mild, Moderate, Severe). Category:\nIntensity:"
MAP  X5, X3

RST  X31                    ; Restore the room state context to combine with the classified temperature feedback for adjustments.

LS   X3, "Temperature user feedback:"
PSH  X3
PSH  X5                     ; Push the user feedback for context.

SNP  X30                    ; Save the temperature feedback context for later adjustment.

; Build the light feedback context for adjustments.
CLR                         ; Clear context stack to classify user light feedback for adjustments.
PSH  X2                     ; Push the user feedback to the context stack for processing.

LS   X3, "Classify the following light feedback into a category (TOO_DARK, TOO_BRIGHT, COMFORTABLE, UNRELATED) and intensity level (Mild, Moderate, Severe). Category:\nIntensity:"
MAP  X6, X3

RST  X31                    ; Restore the room state context to combine with the classified light feedback for adjustments.

LS   X3, "Light intensity user feedback:"
PSH  X3
PSH  X6                     ; Push the light feedback for context.

SNP  X31                    ; Save the light feedback context for later adjustment.

; Think: Adjust the temperature based on the classified feedback.
LI   X29, 5                 ; Set a retry limit to prevent infinite loops in case of invalid adjustments.

RETRY_TEMP:
RST  X30                    ; Restore the temperature feedback context for adjustments.
DEC  X29, 1                 ; Decrement the retry counter.

LI   X3, 0
BEQ  X29, X3, ABORT_TEMP    ; If retry limit is reached, abort the operation.

LS   X3, "Intensity Factor: Mild = 1, Moderate = 2, Severe = 3. If 'TOO_COLD', increase temperature by (0.5 * intensity_factor). If 'TOO_WARM', decrease temperature by (0.5 * intensity_factor). If 'COMFORTABLE', no change to temperature. If 'UNRELATED', no change to temperature.\nWhat is the new room temperature?"
MAP  X7, X3

; Guardrails: Ensure that the temperature adjustments are within safe and reasonable limits.
CLR                         ; Clear context stack to validate the adjusted temperature.
PSH  X7                     ; Push the adjusted temperature for validation.

LS   X3, "Is the temperature mentioned above one of the following: 18°C, 19°C, 20°C, 21°C, 22°C, 23°C, 24°C?"
EVAL X8, X3

LI   X3, 0
BEQ  X8, X3, RETRY_TEMP

; Think: Adjust the light intensity based on the classified feedback.
LI   X29, 5                 ; Set a retry limit to prevent infinite loops in case of invalid adjustments.

RETRY_LIGHT:
RST  X31                    ; Restore the light feedback context for adjustments.
DEC  X29, 1                 ; Decrement the retry counter.

LI   X3, 0
BEQ  X29, X3, ABORT_LIGHT   ; If retry limit is reached, abort the operation.

LS   X3, "Intensity Factor: Mild = 1, Moderate = 2, Severe = 3. If 'TOO_DARK', increase light by (10% * intensity). If 'TOO_BRIGHT', decrease light by (10% * intensity). If 'COMFORTABLE', no change to light. If 'UNRELATED', no change to light.\nWhat is the new light intensity percentage?"
MAP  X9, X3

; Guardrails: Ensure that the light intensity adjustments are within safe and reasonable limits.
CLR                         ; Clear context stack to validate the adjusted light intensity.
PSH  X9                     ; Push the adjusted light intensity for validation.

LS   X3, "Is the light intensity percentage mentioned above between 0% and 100%?"
EVAL X10, X3

LI   X3, 0
BEQ  X10, X3, RETRY_LIGHT

; Act: Implement the adjustments to achieve the desired physical comfort.
CLR                         ; Clear context stack to prepare for output.
PSH  X7                     ; Push the final adjusted temperature for output.
PSH  X9                     ; Push the final adjusted light intensity for output.

LS   X3, "{ \"temp_celsius\": number, \"light_percent\": number }"
MAP  X11, X3

OUT  X11
EXIT

ABORT_TEMP:
LS  X3, "Failed to adjust the room's temperature within the 5 attempts after multiple attempts."
OUT X3
EXIT

ABORT_LIGHT:
LS  X3, "Failed to adjust the room's light intensity within the 5 attempts after multiple attempts."
OUT X3
```

## Registers

There are 32 general-purpose registers, named X1 to X32. These registers can hold text and positive numbers (currently working on support images and audio).

## Context Stack

The context stack is a FILO (First In, Last Out) structure that holds a sequence of messages that the LPU uses to maintain context across multiple instructions. When you push a register onto the context stack, its content is added to the bottom of the stack as a message. When you pop from the context stack, the bottom message is removed and stored in a register. The context stack can be refined during the lifetime of the program, which allows remaining relevant information while discarding irrelevant details.

The instructions `snp`, `rst`, `psh`, `pop`, and `drp` are used to manage the context stack. Whilst `mrf`, `prj`, `dst`, `cor`, and `aud` takes a source register and operates using the context stack as context/previous input. The result of these operations are stored in a destination register.

## Instruction Terminology

- `rd` - destination register
- `rs` - source register
- `imm` - immediate value can be a string or a number
- `label_name` - a label used for branching

## Instruction Set

The instruction set is closely inspired by RISC-V assembly language:

| Instruction | Description                                                                         | Use                        |
| ----------- | ----------------------------------------------------------------------------------- | -------------------------- |
| LS          | Load string into rd                                                                 | `ls rd, "example"`         |
| LI          | Load immediate into rd                                                              | `li rd, imm`               |
| LF          | Load file into rd                                                                   | `lf rd, "file_path"`       |
| MV          | Copy rs into rd                                                                     | `mv rd, rs`                |
| BEQ         | Go to label if rs1 = rs2                                                            | `beq rs1, rs2, label_name` |
| BLT         | Go to label if rs1 < rs2                                                            | `blt rs1, rs2, label_name` |
| BLE         | Go to label if rs1 <= rs2                                                           | `ble rs1, rs2, label_name` |
| BGT         | Go to label if rs1 > rs2                                                            | `bgt rs1, rs2, label_name` |
| BGE         | Go to label if rs1 >= rs2                                                           | `bge rs1, rs2, label_name` |
| CLR         | Clear the context stack                                                             | `clr`                      |
| SNP         | Save the current state to the context stack and store in rd                         | `snp rd`                   |
| RST         | Restore the state from rs in the context stack                                      | `rst rs`                   |
| PSH         | Push rs into the context stack                                                      | `psh rs`                   |
| POP         | Pop the bottom of the context stack into rd                                         | `pop rd`                   |
| DRP         | Drop the bottom of the context stack                                                | `drp`                      |
| SRL         | Set the role of the context push                                                    | `srl "user"\|"assistant"`  |
| MAP         | Change the shape to the form of rs and store in rd                                  | `map rd, rs`               |
| EVAL        | Boolean evaluation of the question rs and store in rd (0 = false/no,, 1 = true/yes) | `eval rd, rs`              |
| SIM         | Cosine similarity between rs and rs and store in rd (0 - 100)                       | `sim rd, rs`               |
| LABEL       | Define a label. Required for branching instructions                                 | `label_name:`              |
| OUT         | Print the value of rs                                                               | `out rs\|imm`              |
| DEC         | Decrement the value in rs by num                                                    | `dec rd, num`              |
| EXIT        | Exit the program                                                                    | `exit`                     |

## Smaller Models

A pain point of working with smaller models (below 2.6B) is that the attention heads are simply not deep enough to map complex relationships between words. They function much closer to advanced autocomplete engines looking for patterns in the input text. Certain words or phrases can steer outcome more than others, which means that the model might completely ignore some words or phrases.

Keep in mind that the smaller the model you choose, the more precise you need to be with your instructions and guardrails. They lack reasoning capabilities, but are good at pattern matching at speed.

> Decompose your reasoning instructions into small simple explicit sequential steps. \
> **KISS (Keep It Simple, Stupid).**

Some tips for working with smaller models:

1. Always tell it exactly what to do, not what not to do.
2. Don't leave any room for interpretation.
3. Keep the instructions and guardrails as simple and straightforward as possible. Keep it strict, and structured like `User_State: Happy \n Room_State: Cold`.
4. Keep the context stack as clean and relevant as possible. Don't push anything that is not directly relevant to the current instruction.

## Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/HuyNguyenAu/language_processor_unit.git
   cd language_processor_unit
   ```
2. Install [llama.cpp](https://github.com/ggml-org/llama.cpp).
3. Download [LFM2 2.6B](https://huggingface.co/LiquidAI/LFM2-2.6B-GGUF).
4. Download [Qwen3 Embedding 0.6B](https://huggingface.co/Qwen/Qwen3-Embedding-0.6B-GGUF).
5. Create the `.env` file in the root directory with the following content:

   ```
   # File name of the text model in the models directory.
   TEXT_MODEL="LFM2-2.6B-Q5_K_M"

   # File name of the embedding model in the models directory.
   EMBEDDING_MODEL="Qwen3-Embedding-0.6B-Q4_1-imat"

   # When true, output byte code of built assembly file.
   DEBUG_BUILD=false

   # When true, output excuted instructions and their results.
   DEBUG_RUN=false
   ```

### With Embeddings Model (Recommended)

6. Start the LLama.cpp server. Make sure to specify the `--embeddings` flag and the correct pooling strategy:
   ```bash
   ./llama-server -np 1 --embeddings --pooling mean --models-dir C:\llama\models
   ```

### Without Embeddings Model (Faster)

6. Start the LLama.cpp server. This will use the text model for both text generation and embeddings, which is faster but less accurate for embeddings. Make sure to specify the `--embeddings` flag and the correct pooling strategy:
   ```bash
   ./llama-server -np 1 --embeddings --pooling mean -m C:\llama\models\LFM2.5-1.2B-Instruct-Q8_0.gguf
   ```

### Run The Example Program

7. Build the example program:
   ```bash
   cargo run build examples/room-comfort.aasm
   ```
8. Run the example program:
   ```bash
   cargo run run build/room-comfort.lpu
   ```

## Acknowledgements

This project was inspired by the following works:

- [Crafting Interpreters](https://craftinginterpreters.com/) by Bob Nystrom. The structure and design of the assembler and processor follows a similar approach to the one described in this book.
- [Andrej Karpathy](https://karpathy.ai/) LLM OS and Software 2.0 ideas.
