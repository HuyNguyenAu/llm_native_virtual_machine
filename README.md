# LLM Native Virtual Machine

Let's start out with an idea _because I'm bored_: **What if a programming language was powered by an LLM?**

Basically, we shifted from having deterministic operations to one that is probabilistic and generative. This new paradigm called "Soft Computing" allows us to work with data that is unstructured, messy, or subjective in a way that traditional computing struggles with. In short, we can handle ambiguity and fuzzy logic, which is becoming more common in real‑world applications.

This project explores the idea of implementing a simple and reduced programming language which attempts to make the programmer focus more on the "what" rather than the "how" by keeping the instructions simple and moving the complexity to the language model.

What we have here is a new way to decompose large complex prompts into small atomic instructions that can be executed sequentially while maintaining a context stack to keep track of the conversation history and relevant information. This allows the language model acheive better performance and accuracy, especially when working with smaller models that have less attention capacity.

Think of this language as a middle ground between traditional programming languages and natural language prompts, where we can write code that is more structured and modular than natural language prompts, but still allows us to work with multi-modal data (only text for now) in a way that is more intuitive and flexible than traditional programming languages.

## Why?

I really wanted to imagine a future where we can write code where we don't have to worry about edge cases or complex logic to handle unstructured data. Instead, we can just write code that describes what we want to achieve, and let the language model handle the complexity of how to achieve it. In short, **we can write code that is more focused on the "what" rather than the "how"**.

Here's an example of what a program written in this language looks like:

```
; Program: Room Comfort Adjustment System
; Objective: Adjust the room's temperature and lighting based on sensor data to achieve optimal physical comfort.
; Output: Adjusted temperature and lighting settings.

; Registers:
; X1: Sensor data (JSON format)
; X2: User feedback (string)
; X3: Temporary register for constructing prompts
; X4: Current temperature information extracted from sensor data
; X5: Classified category of temperature feedback
; X6: Classified intensity of temperature feedback
; X7: Current light intensity information extracted from sensor data
; X8: Classified category of light feedback
; X9: Classified intensity of light feedback
; X10: Adjusted temperature based on feedback
; X11: Validation result for adjusted temperature
; X12: Adjusted light intensity based on feedback
; X13: Output JSON containing the final adjusted temperature and light intensity
; C1: Context stack for sensor data
; C10: Context stack for processing feedback and adjustments
; C30: Saved context for temperature feedback
; C31: Saved context for light feedback

; Load sensor data and user feedback.
LC   X1, "examples/data/room_sensor_data.json"
LS   X2, "It's too dark to read and I am sweating."

PSH  C1, X1, "user"                     ; Push the sensor data the context stack for processing.

; Sense: Build the temperature feedback context for adjustments.
LS   X3, "A sentence that describes the current state of the room with only the temperature information in celsius."
INF  X4, X3, C1

LS   X3, "User feedback:"
PSH  C10, X3, "user"
PSH  C10, X2, "user"                     ; Push the user feedback to the context stack for processing.

LS   X3, "If the user feedback is related to temperature, classify the feedback into one of the following categories: TOO_COLD, TOO_WARM, COMFORTABLE, UNRELATED. Category:"
INF  X5, X3, C10

LS   X3, "If the user feedback is related to temperature, classify the intensity of the feedback into one of the following levels: Mild, Moderate, Severe. Intensity:"
INF  X6, X3, C10

MVC  C10, C0                            ; Clear context stack to classify the temperature feedback.

LS   X3, "Current room state:"
PSH  C10, X3, "user"
PSH  C10, X4, "user"                    ; Push the temperature information for context.

LS   X3, "Category and intensity of the temperature feedback:"
PSH  C10, X3, "user"
PSH  C10, X5, "user"                    ; Push the temperature feedback category for context.
PSH  C10, X6, "user"                    ; Push the temperature feedback intensity for context.

MVC C30, C10                            ; Save the temperature feedback context for later adjustment.

; Sense: Build the light feedback context for adjustments.
LS   X3, "A sentence that describes the current state of the room with only the light intensity information in percentage."
INF  X7, X3, C1

MVC  C10, C0                            ; Clear context stack to classify the light feedback.

LS   X3, "User feedback:"
PSH  C10, X3, "user"
PSH  C10, X2, "user"                    ; Push the user feedback to the context stack for processing.

LS   X3, "If the user feedback is related to light, classify the feedback into one of the following categories: TOO_DARK, TOO_BRIGHT, COMFORTABLE, UNRELATED. Category:"
INF  X8, X3, C10

LS  X3, "If the user feedback is related to light, classify the intensity of the feedback into one of the following levels: Mild, Moderate, Severe. Intensity:"
INF  X9, X3, C10

MVC  C10, C0                            ; Clear context stack to build the light feedback context for adjustments.

LS   X3, "Current room state:"
PSH  C10, X3, "user"
PSH  C10, X7, "user"                    ; Push the light intensity information for context.

LS   X3, "Category and intensity of the light feedback:"
PSH  C10, X3, "user"
PSH  C10, X8, "user"                    ; Push the light feedback category for context.
PSH  C10, X9, "user"                    ; Push the light feedback intensity for context.

MVC  C31, C10                            ; Save the light feedback context for later adjustment.

; Think: Adjust the temperature based on the classified feedback.
LI   X31, 5                             ; Set a retry limit to prevent infinite loops in case of invalid adjustments.

RETRY_TEMP:
MVC  C10, C30                           ; Restore the temperature feedback context for adjustments.
SUBI X31, 1                             ; Decrement the retry counter.

LI   X3, 0
BEQ  X31, X3, ABORT_TEMP                ; If retry limit is reached, abort the operation.

LS   X3, "Intensity Factor: Mild = 1, Moderate = 2, Severe = 3. If 'TOO_COLD', increase temperature by (0.5 * intensity_factor). If 'TOO_WARM', decrease temperature by (0.5 * intensity_factor). If 'COMFORTABLE', no change to temperature. If 'UNRELATED', no change to temperature.\nWhat is the new room temperature?"
INF  X10, X3, C10

; Guardrails: Ensure that the temperature adjustments are within safe and reasonable limits.
MVC  C10, C0                             ; Clear context stack to validate the adjusted temperature.
PSH  C10, X10, "user"                    ; Push the adjusted temperature for validation.

LS   X3, "Is the temperature mentioned above one of the following: 18°C, 18.5°C, 19°C, 19.5°C, 20°C, 20.5°C, 21°C, 21.5°C, 22°C, 22.5°C, 23°C, 23.5°C, 24°C?"
EVAL X11, X3, C10

LI   X3, 0
BEQ  X11, X3, RETRY_TEMP

; Think: Adjust the light intensity based on the classified feedback.
LI   X31, 5                             ; Set a retry limit to prevent infinite loops in case of invalid adjustments.

RETRY_LIGHT:
MVC  C10, C31                           ; Restore the light feedback context for adjustments.
SUBI X31, 1                             ; Decrement the retry counter.

LI   X3, 0
BEQ  X31, X3, ABORT_LIGHT               ; If retry limit is reached, abort the operation.

LS   X3, "Intensity Factor: Mild = 1, Moderate = 2, Severe = 3. If 'TOO_DARK', increase light by (5 * intensity). If 'TOO_BRIGHT', decrease light by (5 * intensity). If 'COMFORTABLE', no change to light. If 'UNRELATED', no change to light.\nWhat is the new light percentage?"
INF  X12, X3, C10

; Guardrails: Ensure that the light intensity adjustments are within safe and reasonable limits.
MVC  C10, C0                             ; Clear context stack to validate the adjusted light intensity.
PSH  C10, X12, "user"                    ; Push the adjusted light intensity for validation.

LS   X3, "Is the light intensity percentage mentioned above between 0% and 100%?"
EVAL X13, X3, C10

LI   X3, 0
BEQ  X13, X3, RETRY_LIGHT

; Act: Implement the adjustments to achieve the desired physical comfort.
MVC  C10, C0                             ; Clear context stack to prepare for output.
PSH  C10, X10, "user"                    ; Push the final adjusted temperature for output.
PSH  C10, X12, "user"                    ; Push the final adjusted light intensity for output.

LS   X3, "{ \"temp_celsius\": number, \"light_percent\": number }"
INF  X13, X3, C10

PLN X13
EXIT

ABORT_TEMP:
LS   X3, "Failed to adjust the room's temperature within the 5 attempts after multiple attempts."
PLN  X3
EXIT

ABORT_LIGHT:
LS   X3, "Failed to adjust the room's light intensity within the 5 attempts after multiple attempts."
PLN  X3
```

## Future Directions

In a way, this project is essentially a low-level language for Agents. Frameworks like LangChain or AutoGPT are great for quickly building agents and applications, but they lack the flexibility and control that this language can provide.

The difference is comparable to Machine Code vs Web Framework. Here we have a fundermental execution engine or model that has the following distinct advantages:

1. Abstraction: We operate on atomic instructions. If you want to move data, you use `MV`. If you want to check a condition, you `EVAL` then `BEQ`. There is no black box logic.
2. Context/Memory Management: We have context registers which allow for fine-grained control over what information is relevant for each instruction.
3. Control Flow: The LLM only handles data processing and generation, while the control flow is handled by the assembler. The decision of where to go next is handled by branching. This is more robust since the logic of the program is not inside a prompt.

A dream state for this project would be to have other frameworks and languages compile down to this language to provide a transparent, auditable sequence of operations that can be optimised for cost and speed.

## Registers

There are 33 general-purpose registers, named X0 to X32. These registers can hold text and positive numbers (currently working on support images and audio). Register X0 is a special read-only register that always holds the value 0.

Similary, there are 33 context registers, named C0 to C32. These registers are used to manage the context stack, which is a FILO (First In, Last Out) structure that holds a sequence of messages that certain instructions use to maintain context. The context register C0 is a special read-only register that always holds an empty context stack.

## Context Stack

The context stack is a FILO (First In, Last Out) structure that holds a sequence of messages that certain instructions use to maintain context. When you push a register onto the context stack, its content is added to the bottom of the stack as a message. When you pop from the context stack, the bottom message is removed and stored in a register. The context stack can be refined during the lifetime of the program, which allows remaining relevant information while discarding irrelevant details.

The instructions `MVC`, `PSH`, `POP`, and `DRP` are used to manage the context stack. `GEN` creates a model response prompt, and `EVAL` takes the question/query from the source register and evaluates it as a boolean question. Both of these instructions use the context stack previous history. This means that you can refine and manage the context stack to improve performance for the `MAP` and `EVAL` instructions, which is especially important when working with smaller models that have less attention capacity.

## Instruction Terminology

- `rd` - destination general-purpose register
- `rs` - source general-purpose register
- `rdc` - destination context register
- `rsc` - source context register
- `imm` - number value
- `str` - string value
- `label_name` - a label used for branching

## Instruction Set

The instruction set is loosely inspired by RISC-V assembly language:

| Instruction | Description                                                                                                                      | Use                                |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| LS          | Load string into `rd`                                                                                                            | `ls rd, str`                       |
| LI          | Load immediate into `rd`                                                                                                         | `li rd, imm`                       |
| LC          | Load the content from the path `rs` into `rd`                                                                                    | `lc rd, str`                       |
| MV          | Copy `rs` into `rd`                                                                                                              | `mv rd, rs`                        |
| MVC         | Copy `rsc` into `rdc`                                                                                                            | `mvc rdc, rsc`                     |
| BEQ         | Go to label if `rs1` = `rs2`                                                                                                     | `beq rs1, rs2, label_name`         |
| BNE         | Go to label if `rs1` != `rs2`                                                                                                    | `bne rs1, rs2, label_name`         |
| BLT         | Go to label if `rs1` < `rs2`                                                                                                     | `blt rs1, rs2, label_name`         |
| BLE         | Go to label if `rs1` <= `rs2`                                                                                                    | `ble rs1, rs2, label_name`         |
| BGT         | Go to label if `rs1` > `rs2`                                                                                                     | `bgt rs1, rs2, label_name`         |
| BGE         | Go to label if `rs1` >= `rs2`                                                                                                    | `bge rs1, rs2, label_name`         |
| PSH         | Push `rs` into the context stack `rdc` with role                                                                                 | `psh rdc, rs, "user"\|"assistant"` |
| POP         | Pop the bottom of the context stack `rsc` into `rd`                                                                              | `pop rd, rsc`                      |
| DRP         | Drop the bottom of the context stack `rsc`                                                                                       | `drp rsc`                          |
| INF         | Use `rs` as the next message and store the response in `rd` using context register `rsc`                                         | `inf rd, rs, rsc`                  |
| EVAL        | Boolean evaluation of the question `rs` and store the response in `rd` (0 = false/no, 1 = true/yes) using context register `rsc` | `eval rd, rs, rsc`                 |
| SIM         | Cosine similarity between `rs` and `rs` and store the result in `rd` (0 - 100)                                                   | `sim rd, rs`                       |
| RLN         | Read from `rs1` with line index `rs2` and store line in `rd`                                                                     | `rln rd, rs1, rs2`                 |
| CLN         | Get the line count of `rs` and store in `rd`                                                                                     | `cln rd, rs`                       |
| PUT         | Print the value of `rs`                                                                                                          | `put rs`                           |
| PLN         | Print the value of `rs` followed by a newline                                                                                    | `pln rs`                           |
| PCX         | Print the content of the context register `rsc`                                                                                  | `pcx rsc`                          |
| ADDI        | Increment the value in `rs` by `num`                                                                                             | `addi rd, num`                     |
| SUBI        | Decrement the value in `rs` by `num`                                                                                             | `subi rd, num`                     |
| LABEL       | Define a label. Required for branching instructions                                                                              | `label_name:`                      |
| EXIT        | Exit the program                                                                                                                 | `exit`                             |

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
3. Download [LFM2 2.6B](https://huggingface.co/LiquidAI/LFM2-2.6B-GGUF). [LFM2 8B-A1B](https://huggingface.co/LiquidAI/LFM2-8B-A1B-GGUF) is also a very good option if you have the hardware for it.
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

   # When true, output chat interactions with the language model.
   DEBUG_CHAT=false

   # LFM2-2.6B recommended parameters.
   TEXT_MODEL_TEMPERATURE=0.3
   TEXT_MODEL_MIN_P=0.15
   TEXT_MODEL_REPEAT_PENALTY=1.05
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

### Why Use LFM2 models?

LFM2 models are very fast and capable enough for general purpose tasks relative to their size and have decent knowledge and reasoning capabilities. Here we are more concerned with the speed of the model because the LPU is designed to work with smaller models that can run on consumer hardware.

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
