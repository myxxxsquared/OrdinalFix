import os
import javalang
import random
import collections

# add, remove, replace, duplicate   punctuations  { } ( ) ; , . = ==  keywords return if null new
# add, remove, duplicate identifier
# replace identifier

# input_file = "right/0e97a13e3ca3a86f2081406d801e15a7f05ed95f/block"

# with open(input_file, "rt") as fin:
#     tokens = fin.read()
# tokens = javalang.tokenizer.tokenize(tokens)
# tokens = [token.value for token in tokens]

# m_p_{1,2,3,4,5,6,7,8,9,10}
# m_pi_{1,2,3,4,5,6,7,8,9,10}
# m_i_{1,2,3,4,5,6,7,8,9,10}
# m_a_{1,2,3,4,5,6,7,8,9,10}


def main():
    # punctuations = {"{", "}", "(", ")", ";", ",", ".", "=", "==", "return", "if", "null", "new", "else"}
    # object_name = "Object"
    # prefixes = ["CLASS_", "FIELD_", "METHOD_", "VAR_"]
    mutant = Mutant(0)

    for subfolder in os.listdir("right"):
        identifiers = []
        with open(f"right/{subfolder}/env", "rt") as fin:
            tokens = fin.read()
        block_content = tokens
        tokens = javalang.tokenizer.tokenize(tokens)
        identifiers.extend(token.value for token in tokens if isinstance(token, javalang.tokenizer.Identifier))

        with open(f"right/{subfolder}/block", "rt") as fin:
            tokens = fin.read()
        tokens = list(javalang.tokenizer.tokenize(tokens))
        identifiers.extend(token.value for token in tokens if isinstance(token, javalang.tokenizer.Identifier))
        tokens = [token.value if token.value != "CLASS_Object" else "Object" for token in tokens]
        identifiers = sorted(list(set(identifiers)))
        identifier_map = collections.defaultdict(list)
        for identifier in identifiers:
            prefix = identifier.split("_")[0]
            identifier_map[prefix].append(identifier)

        for mutant_type in ["p", "pi", "i", "a"]:
            for mutant_count in range(1, 11):
                try:
                    new_tokens = mutant.mutant(tokens, identifiers, identifier_map, mutant_type, mutant_count)
                    new_tokens = [token if token != "CLASS_Object" else "Object" for token in new_tokens]
                    new_tokens = "\n".join(new_tokens)
                    os.makedirs(f"m_{mutant_type}_{mutant_count}/{subfolder}", exist_ok=True)
                    with open(f"m_{mutant_type}_{mutant_count}/{subfolder}/env", "wt") as fout:
                        fout.write(block_content)
                    with open(f"m_{mutant_type}_{mutant_count}/{subfolder}/block", "wt") as fout:
                        fout.write(new_tokens)
                except InvalidMutantExcpetion:
                    pass


class InvalidMutantExcpetion(Exception):
    pass


class Mutant:
    PUNCTUATIONS = ["{", "}", "(", ")", ";", ",", ".", "=", "=="]
    KEYWORDS = ["return", "if", "null", "new", "else"]
    ALLINSERT = PUNCTUATIONS + KEYWORDS
    ALLINSERT_SET = set(ALLINSERT)

    def __init__(self, seed):
        self.random = random.Random(seed)

    def mutant(self, tokens, identifiers, identifier_map, mutant_type, mutant_count):
        mutant_operators = {
            "p": [self.mutant_punctuation_add, self.mutant_punctuation_remove, self.mutant_punctuation_replace, self.mutant_punctuation_duplicate],
            "pi": [self.mutant_identifier_duplicate, self.mutant_identifier_add, self.mutant_identifier_remove],
            "i": [self.mutant_identifier_replace],
            "a": [
                self.mutant_punctuation_add,
                self.mutant_punctuation_remove,
                self.mutant_punctuation_replace,
                self.mutant_punctuation_duplicate,
                self.mutant_identifier_duplicate,
                self.mutant_identifier_add,
                self.mutant_identifier_remove,
                self.mutant_identifier_replace,
            ],
        }[mutant_type]

        original_tokens = tokens
        last_ex = None
        for _ in range(5):
            tokens = list(original_tokens)
            try:
                for _ in range(mutant_count):
                    tokens = mutant_operators[self.random.randint(0, len(mutant_operators) - 1)](tokens, identifiers, identifier_map)
                return tokens
            except InvalidMutantExcpetion as ex:
                last_ex = ex
        raise last_ex

    def mutant_punctuation_add(self, tokens, identifiers, identifier_map):
        loc = self.random.randint(0, len(tokens))
        token = self.random.sample(Mutant.ALLINSERT, 1)[0]
        tokens = list(tokens)
        tokens.insert(loc, token)
        return tokens

    def mutant_punctuation_remove(self, tokens, identifiers, identifier_map):
        locs = [i for i, t in enumerate(tokens) if t in Mutant.ALLINSERT_SET]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        loc = locs[self.random.randint(0, len(locs) - 1)]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        tokens = list(tokens)
        tokens.pop(loc)
        return tokens

    def mutant_punctuation_replace(self, tokens, identifiers, identifier_map):
        locs = [i for i, t in enumerate(tokens) if t in Mutant.ALLINSERT_SET]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        loc = locs[self.random.randint(0, len(locs) - 1)]
        old_token = tokens[loc]
        new_token = self.random.sample(Mutant.ALLINSERT, 1)[0]
        while new_token == old_token:
            new_token = self.random.sample(Mutant.ALLINSERT, 1)[0]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        tokens[loc] = new_token
        tokens.pop(loc)
        return tokens

    def mutant_punctuation_duplicate(self, tokens, identifiers, identifier_map):
        locs = [i for i, t in enumerate(tokens) if t in Mutant.ALLINSERT_SET]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        loc = locs[self.random.randint(0, len(locs) - 1)]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        tokens = list(tokens)
        tokens.insert(loc, tokens[loc])
        return tokens

    def mutant_identifier_duplicate(self, tokens, identifiers, identifier_map):
        locs = [i for i, t in enumerate(tokens) if t not in Mutant.ALLINSERT_SET]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        loc = locs[self.random.randint(0, len(locs) - 1)]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        tokens = list(tokens)
        tokens.insert(loc, tokens[loc])
        return tokens

    def mutant_identifier_add(self, tokens, identifiers, identifier_map):
        loc = self.random.randint(0, len(tokens))
        token = self.random.sample(identifiers, 1)[0]
        tokens = list(tokens)
        tokens.insert(loc, token)
        return tokens

    def mutant_identifier_remove(self, tokens, identifiers, identifier_map):
        locs = [i for i, t in enumerate(tokens) if t not in Mutant.ALLINSERT_SET]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        loc = locs[self.random.randint(0, len(locs) - 1)]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        tokens = list(tokens)
        tokens.pop(loc)
        return tokens

    def mutant_identifier_replace(self, tokens, identifiers, identifier_map):
        locs = [i for i, t in enumerate(tokens) if t not in Mutant.ALLINSERT_SET]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        loc = locs[self.random.randint(0, len(locs) - 1)]
        old_token = tokens[loc]
        token_prefix = old_token.split("_")[0]
        new_token = self.random.sample(list(identifiers), 1)[0]
        while new_token == old_token:
            new_token = self.random.sample(list(identifiers), 1)[0]
        if len(locs) == 0:
            raise InvalidMutantExcpetion()
        tokens[loc] = new_token
        tokens.pop(loc)
        return tokens


if __name__ == "__main__":
    main()
