import csv
import dataclasses
import json
import multiprocessing
import os
import re
import shutil
import subprocess
import traceback
import typing

import tqdm

import deepfixutlis
import pycparser

TMP_FOLDER = "tmp/"
PREPROCESSOR = "gcc"
# PREPROCESSOR = "clang"
FAKE_INCLUDE_PATH = "pycparser/utils/fake_libc_include"
FUNCS_INCLUDE_PATH = "c_include_contain_funcs"
PREPROCESSOR_FLAGS = ["-E", "-I", FUNCS_INCLUDE_PATH, "-I", FAKE_INCLUDE_PATH]
OUTPUT_FOLDER = "checkout/processed/"
PRE_PROCESS_INPUT_FILE_NAME = "__cgrammar_preprocess_input.c"

COMPILER = "gcc"
COMPILER_FLAGS = ["-std=c99", "-c"]
COMPILE_INPUT_FILE_NAME = "__cgrammar_compile_input.c"
COMPILE_OUTPUT_FILE_NAME = "__cgrammar_compile_input.o"

RE_LINEMARKER = re.compile(r"#\s*(\d+)\s+" + r'"([^"]+)"' + r"\s*(\d+(?:\s+\d+)*)?\s*")

NUM_PROCESSES = 32

ST = {
    "ID": "IDENTIFIER",
    "INT_CONST_DEC": "LITERAL_INT",
    "INT_CONST_OCT": "LITERAL_INT",
    "INT_CONST_HEX": "LITERAL_INT",
    "INT_CONST_BIN": "LITERAL_INT",
    "INT_CONST_CHAR": "LITERAL_INT",
    "FLOAT_CONST": "LITERAL_FLOAT",
    "HEX_FLOAT_CONST": "LITERAL_FLOAT",
    "CHAR_CONST": "LITERAL_INT",
    "WCHAR_CONST": "LITERAL_INT",
    "STRING_LITERAL": "LITERAL_STRING",
    "WSTRING_LITERAL": "LITERAL_STRING",
}


class ErrorWriter:
    file: typing.TextIO
    name: str

    def __init__(self):
        self.file = None
        self.name = None

    def write(self, text: str):
        if self.file is None:
            self.file = open(self.name, "w")
        self.file.write(text + "\n")

    def new_file_start(self, name: str):
        if self.file is not None:
            self.file.close()
            self.file = None
            self.name = None
        self.name = name


def preprocess_file(file_name: str, err: ErrorWriter):
    pid = os.getpid()
    working_dir = f"{TMP_FOLDER}{pid}/"
    os.makedirs(working_dir, exist_ok=True)
    sample_file_input = f"{working_dir}{PRE_PROCESS_INPUT_FILE_NAME}"
    sample_file_output = f"{working_dir}preprocess_output.c"
    if os.path.exists(sample_file_input):
        os.remove(sample_file_input)
    if os.path.exists(sample_file_output):
        os.remove(sample_file_output)
    shutil.copy(file_name, sample_file_input)
    result = subprocess.run(
        [
            PREPROCESSOR,
            *PREPROCESSOR_FLAGS,
            sample_file_input,
            "-o",
            sample_file_output,
        ],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        if err is not None:
            err.write(
                "FAILED TO PROPRESS:\nSTDOUT: {}\nSTDERR: {}".format(
                    result.stdout.decode("utf-8", errors="ignore"),
                    result.stderr.decode("utf-8", errors="ignore"),
                ),
            )
        return "failed_preprocess"
    if not os.path.exists(sample_file_output):
        if err is not None:
            err.write(
                f"FAILED TO PROPRESS:\nNO OUTPUT FILE",
            )
        return "failed_preprocess_no_output"
    with open(sample_file_output) as fin:
        code_text = fin.readlines()
    return code_text


def preprocess(
    sample: deepfixutlis.DeepFixSample, err: ErrorWriter, output_folder: str
):
    output_original_file = f"{output_folder}original.c"
    with open(output_original_file, "w") as fout:
        fout.write(sample.code)
    if try_compile(output_original_file):
        return "original_compiles"
    code_text = preprocess_file(output_original_file, err)
    if isinstance(code_text, str):
        return code_text
    with open(f"{output_folder}preprocessed.c", "w") as fout:
        fout.writelines(code_text)
    return code_text


Token = typing.NamedTuple("Token", [("type", str), ("name", str), ("value", str)])


@dataclasses.dataclass
class BraceInProgram:
    content: list[Token]
    inserted_loc: int


@dataclasses.dataclass
class CodeSegment:
    file: str
    line: int
    flags: list[int]
    begin_line: str
    is_original: bool
    code: list[str]
    tokens: list[Token]
    truncated_tokens: list[Token]
    braces: list[BraceInProgram]
    include_line: typing.Optional[str]


def split_code(code_text: list[str]) -> list[CodeSegment]:
    appending = None
    in_original_file = False
    result = []
    known_include_files = {
        "stdio.h",
        "stdlib.h",
        "string.h",
        "math.h",
        "limits.h",
        "strings.h",
        "ctype.h",
    }
    for line in code_text:
        if line.startswith("#"):
            if appending is not None and (appending.code or appending.include_line):
                result.append(appending)
            match = RE_LINEMARKER.match(line)
            if match is None:
                raise ValueError(f"Failed to parse line marker: {line}")
            line_number = int(match.group(1))
            file_name = match.group(2)
            flags = [int(x) for x in match.group(3).split()] if match.group(3) else []
            is_original = PRE_PROCESS_INPUT_FILE_NAME in file_name
            include_line = None
            if in_original_file and not is_original and file_name != "<built-in>":
                if not file_name.startswith(
                    FAKE_INCLUDE_PATH
                ) and not file_name.startswith(FUNCS_INCLUDE_PATH):
                    print("NOT FILE NAME IN FAKE INCLUDE.", file_name)
                else:
                    if file_name.startswith(FUNCS_INCLUDE_PATH):
                        rel_file_name = file_name[len(FUNCS_INCLUDE_PATH) + 1 :]
                    elif file_name.startswith(FAKE_INCLUDE_PATH):
                        rel_file_name = file_name[len(FAKE_INCLUDE_PATH) + 1 :]
                    else:
                        raise ValueError(f"Invalid file name: {file_name}")
                    if rel_file_name not in known_include_files:
                        print("NEW INCLUDE FILE", rel_file_name)
                    include_line = f'#include "{rel_file_name}"'
            appending = CodeSegment(
                file=file_name,
                line=line_number,
                flags=flags,
                begin_line=line,
                is_original=is_original,
                code=[],
                tokens=None,
                truncated_tokens=None,
                braces=None,
                include_line=include_line,
            )
            in_original_file = is_original
        else:
            appending.code.append(line)
    if appending is not None and appending.code:
        result.append(appending)
    return result


def tokenize_segment(code: CodeSegment, identifer_set: set[str]):
    lexer = pycparser.c_lexer.CLexer(
        lambda x, y, z: None, lambda: None, lambda: None, lambda x: False
    )
    lexer.build()
    code_text = "".join(code.code)
    lexer.input(code_text)
    tokens = []
    while True:
        next_token = lexer.token()
        if next_token is None:
            break
        next_token_type = next_token.type
        next_token_value = next_token.value
        if next_token_type == "ID":
            identifer_set.add(next_token_value)
        if next_token_type in ST:
            tokens.append(Token("ST", ST[next_token_type], next_token_value))
        else:
            tokens.append(Token("LT", next_token_value, next_token_value))
    code.tokens = tokens


def tokenize_all(segments: list[CodeSegment]) -> set[str]:
    identifer_set = set()
    for segment in segments:
        tokenize_segment(segment, identifer_set)
    return identifer_set


def find_braces(code: CodeSegment) -> bool:
    if not code.is_original:
        code.truncated_tokens = code.tokens
        code.braces = []
        return True

    braces = []
    truncated_tokens = []
    cur_depth = 0
    brace_start = None
    for i, t in enumerate(code.tokens):
        if t.name == "{":
            if cur_depth == 0:
                brace_start = i
                truncated_tokens.append(t)
            cur_depth += 1
        elif t.name == "}":
            cur_depth -= 1
            if cur_depth == 0:
                braces.append(
                    BraceInProgram(
                        content=code.tokens[brace_start + 1 : i],
                        inserted_loc=len(truncated_tokens),
                    )
                )
                truncated_tokens.append(t)
            elif cur_depth < 0:
                if len(braces) == 0:
                    return False
                cur_depth = 0
                last_brace = braces[-1]
                braces[-1].content.extend(truncated_tokens[last_brace.inserted_loc : i])
                truncated_tokens = truncated_tokens[: last_brace.inserted_loc]
                truncated_tokens.append(t)
        else:
            if cur_depth == 0:
                truncated_tokens.append(t)
    if cur_depth != 0:
        braces.append(
            BraceInProgram(
                content=code.tokens[brace_start + 1 :],
                inserted_loc=len(truncated_tokens),
            )
        )
        truncated_tokens.append(Token("ST", "}", "}"))

    code.truncated_tokens = truncated_tokens
    code.braces = braces

    return True


def find_braces_all(segments: list[CodeSegment], err: ErrorWriter) -> bool:
    for segment in segments:
        if not find_braces(segment):
            all_code = "".join(segment.code)
            err.write(f"FAILED TO FIND BRACES IN\n{all_code}")
            return False
    return True


def check_real_func_body(segments: list[CodeSegment], err: ErrorWriter) -> str:
    begin_idxes = []
    token_strs = []
    for i, code in enumerate(segments):
        begin_idxes.append(len(token_strs))
        token_strs.extend([t.value for t in code.truncated_tokens])
    token_strs = "\n".join(token_strs)
    parser = pycparser.CParser()
    try:
        ast = parser.parse(token_strs)
    except pycparser.plyparser.ParseError as e:
        exc = traceback.format_exc()
        err.write(f"FAILED TO PARSE\n{exc}\n{token_strs}")
        return "failed_to_parse"
    c_ast = pycparser.c_parser.c_ast
    real_func_loc = set()
    for ext in ast.ext:
        if isinstance(ext, c_ast.Typedef):
            pass
        elif isinstance(ext, c_ast.FuncDef):
            func_body_line = ext.body.coord.line
            seg_idx = None
            for i, seg in enumerate(begin_idxes):
                if seg >= func_body_line:
                    seg_idx = i - 1
                    break
            if seg_idx is None:
                seg_idx = len(segments) - 1
            line_in_seg = func_body_line - begin_idxes[seg_idx]
            real_func_loc.add((seg_idx, line_in_seg))
        elif isinstance(ext, c_ast.Decl):
            pass
        else:
            print(type(ext))

    for i, seg in enumerate(segments):
        if not seg.is_original or len(seg.braces) == 0:
            continue
        real_funcs = []
        insert_back = []
        for brace in seg.braces:
            if (i, brace.inserted_loc) in real_func_loc:
                real_funcs.append(brace)
            else:
                insert_back.append(brace)
        if len(insert_back) == 0:
            continue
        truncated_tokens = []
        next_to_insert = 0
        for i in insert_back:
            truncated_tokens.extend(
                seg.truncated_tokens[next_to_insert : i.inserted_loc]
            )
            truncated_tokens.extend(i.content)
            next_to_insert = i.inserted_loc
            for r in real_funcs:
                if r.inserted_loc > i.inserted_loc:
                    r.inserted_loc += len(i.content)
        truncated_tokens.extend(seg.truncated_tokens[next_to_insert:])
        seg.truncated_tokens = truncated_tokens
        seg.braces = real_funcs
    return None


def try_compile(file_name: str) -> bool:
    pid = os.getpid()
    working_dir = f"{TMP_FOLDER}{pid}/"
    os.makedirs(working_dir, exist_ok=True)
    compile_input_file_name = f"{working_dir}{COMPILE_INPUT_FILE_NAME}"
    compile_output_file_name = f"{working_dir}{COMPILE_OUTPUT_FILE_NAME}"
    os.system(f"rm -rf {compile_input_file_name} {compile_output_file_name}")

    shutil.copy(file_name, compile_input_file_name)
    result = subprocess.run(
        [
            COMPILER,
            *COMPILER_FLAGS,
            compile_input_file_name,
            "-o",
            compile_output_file_name,
        ],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    succ = False
    with open(f"{file_name}.result", "w") as fout:
        if result.returncode == 0:
            if os.path.exists(compile_output_file_name):
                fout.write("RESULT:OK\n")
                succ = True
            else:
                fout.write("RESULT:OK_NO_FILE\n")
        else:
            fout.write("RESULT:FAIL\n")
        stdout = result.stdout.decode("utf-8", errors="ignore")
        stderr = result.stderr.decode("utf-8", errors="ignore")
        fout.write(f"STDOUT:\n{stdout}\n")
        fout.write(f"STDERR:\n{stderr}\n")
    return succ


def regenerate_code(
    segments: list[CodeSegment], output_folder: str
) -> list[(str, str)]:
    program_id = 0
    gen_tokens = []
    all_tokens = []
    valid_samples = []
    for i, seg in enumerate(segments):
        if seg.is_original:
            next_to_insert = 0
            for brace in seg.braces:
                new_tokens = list(
                    t.value
                    for t in seg.truncated_tokens[next_to_insert : brace.inserted_loc]
                )
                gen_tokens.extend(new_tokens)
                all_tokens.extend(new_tokens)
                next_to_insert = brace.inserted_loc

                output_env_name = f"{output_folder}env_{program_id}.c"
                with open(output_env_name, "w") as f:
                    f.write("\n".join(gen_tokens))
                    f.write("\n}\n")

                output_tofix_name = f"{output_folder}tofix_{program_id}.c"
                with open(output_tofix_name, "w") as f:
                    for t in brace.content:
                        assert "\n" not in t.value
                        assert "\n" not in t.name
                        assert "\t" not in t.value
                        assert "\t" not in t.name
                    f.write(
                        "\n".join(
                            f"{t.type}\t{t.name}\t{t.value}" for t in brace.content
                        )
                    )

                output_tofix_raw_name = f"{output_folder}tofix_raw_{program_id}.c"
                with open(output_tofix_raw_name, "w") as f:
                    f.write("\n".join(t.value for t in brace.content))

                output_withfunc_name = f"{output_folder}withfunc_{program_id}.c"
                with open(output_withfunc_name, "w") as f:
                    f.write("\n".join(gen_tokens))
                    f.write("\n")
                    f.write("\n".join(t.value for t in brace.content))
                    f.write("\n}\n")
                if not try_compile(output_withfunc_name):
                    valid_samples.append((output_env_name, output_tofix_name))

                all_tokens.extend(t.value for t in brace.content)
                program_id += 1

            new_tokens = list(t.value for t in seg.truncated_tokens[next_to_insert:])
            gen_tokens.extend(new_tokens)
            all_tokens.extend(new_tokens)
        else:
            if seg.include_line is not None:
                gen_tokens.append(seg.include_line)
                all_tokens.append(seg.include_line)
    all_env_name = f"{output_folder}all_env.c"
    with open(all_env_name, "w") as f:
        f.write("\n".join(gen_tokens))
    if not try_compile(all_env_name):
        return "env_not_compile"
    all_tokens_name = f"{output_folder}all_tokens.c"
    with open(all_tokens_name, "w") as f:
        f.write("\n".join(all_tokens))
    if try_compile(all_tokens_name):
        return "all_tokens_compile"

    return valid_samples


@dataclasses.dataclass
class ProcessResult:
    succ: bool
    code_id: str
    err_reason: str
    valid_samples: list[(str, str)]


def process(sample: deepfixutlis.DeepFixSample) -> ProcessResult:
    err = ErrorWriter()
    output_folder = f"{OUTPUT_FOLDER}{sample.code_id}/"
    os.makedirs(output_folder, exist_ok=True)
    output_error_file = f"{output_folder}err.log"
    err.new_file_start(output_error_file)
    preprocessed = preprocess(sample, err, output_folder)
    if isinstance(preprocessed, str):
        return ProcessResult(
            succ=False,
            code_id=sample.code_id,
            err_reason=preprocessed,
            valid_samples=[],
        )
    splitted = split_code(preprocessed)
    identifer_set = tokenize_all(splitted)
    if not find_braces_all(splitted, err):
        return ProcessResult(
            succ=False,
            code_id=sample.code_id,
            err_reason="find_braces_all",
            valid_samples=[],
        )
    c = check_real_func_body(splitted, err)
    if c:
        return ProcessResult(
            succ=False,
            code_id=sample.code_id,
            err_reason=c,
            valid_samples=[],
        )
    regen = regenerate_code(splitted, output_folder)
    if isinstance(regen, str):
        return ProcessResult(
            succ=False,
            code_id=sample.code_id,
            err_reason=regen,
            valid_samples=[],
        )
    if len(regen) == 0:
        return ProcessResult(
            succ=False,
            code_id=sample.code_id,
            err_reason="no_valid_samples",
            valid_samples=[],
        )
    return ProcessResult(
        succ=True,
        code_id=sample.code_id,
        err_reason="",
        valid_samples=regen,
    )


def main():
    os.system("mkdir -p tmp")
    os.system("mkdir -p checkout")
    os.system(f"rm -rf {OUTPUT_FOLDER}")

    pool = multiprocessing.Pool(processes=NUM_PROCESSES)

    dataset = deepfixutlis.load_all_with_error()
    with open("checkout/result.csv", "w", newline="") as fout:
        writer = csv.writer(fout)
        t = tqdm.tqdm(pool.imap_unordered(process, dataset), total=len(dataset))
        for r in t:
            writer.writerow(
                (
                    r.code_id,
                    r.succ,
                    r.err_reason,
                    json.dumps([list(x) for x in r.valid_samples]),
                )
            )


if __name__ == "__main__":
    main()
