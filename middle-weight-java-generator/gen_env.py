import random
import traceback
import hashlib
import os


class MJCls:
    def __init__(self, name):
        self.name = name
        self.extends = None
        self.fields = None
        self.fields_list = None
        self.constructor = None
        self.methods = None
        self.methods_list = None

        self.idx_descendants = set()
        self.idx_ancestors = set()
        self.idx_childern = set()
        self.idx_field = set()
        self.idx_method = set()
        self.idx_transferable = set()

        self._hash = hash(self.name)

    def __equal__(self, other):
        return self is other

    def __hash__(self):
        return self._hash


class MJFunc:
    def __init__(self, name, params, returns, cls):
        self.name = name
        self.params = params
        self.returns = returns
        self.cls = cls

        self._hash = hash(self.name) + hash(self.cls) * 31

    def __equal__(self, other):
        return self is other

    def __hash__(self):
        return self._hash


class MJField:
    def __init__(self, t, name, cls):
        self.t = t
        self.name = name
        self.cls = cls

        self._hash = hash(self.name) + hash(self.cls) * 31

    def __equal__(self, other):
        return self is other

    def __hash__(self):
        return self._hash


class DistUniform:
    def __init__(self, from_, to):
        self.from_ = from_
        self.to = to

    def __call__(self, r):
        return r.randint(self.from_, self.to)


class CustomListDist:
    def __init__(self, values):
        self.values = values

    def __call__(self, r):
        return self.values[r.randint(0, len(self.values) - 1)]


class MJEnv:
    def __init__(self, clazzes):
        self.clazzes_list = clazzes
        self.clazzes = {clazz.name: clazz for clazz in clazzes}
        self.__construct_idxes()

    def __construct_idxes(self):
        for clazz in self.clazzes_list:
            if clazz.extends is not None:
                clazz.extends.idx_childern.add(clazz)

                cur = clazz.extends
                while cur is not None:
                    cur.idx_descendants.add(clazz)
                    clazz.idx_ancestors.add(cur)
                    cur = cur.extends
        for clazz in self.clazzes_list:
            clazz.idx_transferable = {clazz}.union(clazz.idx_descendants).union(clazz.idx_ancestors)

        for clazz in self.clazzes_list:
            for field in clazz.fields_list:
                field.t.idx_field.add(field)
                for child in field.t.idx_ancestors:
                    child.idx_field.add(field)

            for method in clazz.methods_list:
                if method.returns is not None:
                    method.returns.idx_method.add(method)
                    for child in method.returns.idx_ancestors:
                        child.idx_method.add(method)

    def dump_params(self, params):
        return ", ".join(f"{k.name} {v}" for k, v in params)

    def dump_constructor(self, constructor: MJFunc):
        return f"    {constructor.cls.name} ({self.dump_params(constructor.params)}) {{super();}}\n"

    def dump_func(self, func: MJFunc):
        return_type = "void" if func.returns is None else func.returns.name
        return_stmt = "" if func.returns is None else "return null;"
        return f"    {return_type} {func.name} ({self.dump_params(func.params)}) {{{return_stmt}}}\n"

    def dump_field(self, field: MJField):
        return f"    {field.t.name} {field.name};\n"

    def dump_cls(self, cls: MJCls):
        if cls.name == "Object":
            return ""
        extends_expr = "" if cls.extends is None else f"extends {cls.extends.name} "
        result = [f"class {cls.name} {extends_expr}{{\n"]
        fields = sorted(cls.fields_list, key=lambda x: int(x.name[6:]))
        for field in fields:
            result.append(self.dump_field(field))
        result.append("\n")
        result.append(self.dump_constructor(cls.constructor))
        result.append("\n")
        methods = sorted(cls.methods_list, key=lambda x: int(x.name[7:]))
        for func in methods:
            result.append(self.dump_func(func))
        result.append("}")

        return "".join(result)

    def dump(self):
        clses = sorted(filter(lambda x: x.name != "Object", self.clazzes_list), key=lambda x: int(x.name[6:]))
        return "\n\n".join([self.dump_cls(cls) for cls in clses])


class MJEnvGenerator:
    def __init__(self, seed=0):
        self.random = random.Random(seed)

    def gen_digital(self):
        return f"{self.random.randint(0, 999):03d}"

    def gen_cls_graph(self, num_classes):
        clazzes = set()
        while len(clazzes) < num_classes:
            clazzes.add(f"CLASS_{self.gen_digital()}")
        clazzes = list(clazzes)
        self.random.shuffle(clazzes)
        clazz_object = MJCls("Object")
        clazz_object.fields = dict()
        clazz_object.fields_list = list()
        clazz_object.constructor = MJFunc("CONSTRUCTOR", [], None, clazz_object)
        clazz_object.methods = dict()
        clazz_object.methods_list = list()
        clazzes = [clazz_object] + [MJCls(x) for x in clazzes]
        for i, clazz in enumerate(clazzes):
            if i == 0:
                continue
            clazz.extends = clazzes[self.random.randint(0, i - 1)]
        return clazzes

    def gen_fields(self, clazz, clazzes, num_fields):
        fields = set()
        while len(fields) < num_fields:
            fields.add(f"FIELD_{self.gen_digital()}")
        fields = {x: MJField(self.random.sample(clazzes, 1)[0], x, clazz) for x in fields}
        clazz.fields = fields
        clazz.fields_list = list(fields.values())

    def gen_params(self, clazzes, num_params):
        names = set()
        while len(names) < num_params:
            names.add(f"VAR_{self.gen_digital()}")
        names = [(self.random.sample(clazzes, 1)[0], x) for x in names]
        return names

    def gen_constructor(self, clazz, clazzes, num_params):
        clazz.constructor = MJFunc("CONSTRUCTOR", self.gen_params(clazzes, num_params), None, clazz)

    def gen_methods(self, clazz, clazzes, num_methods, d_num_params, d_returns):
        methods = set()
        while len(methods) < num_methods:
            methods.add(f"METHOD_{self.gen_digital()}")
        methods = {x: MJFunc(x, self.gen_params(clazzes, d_num_params(self.random)), self.random.sample(clazzes, 1)[0] if d_returns(self.random) else None, clazz) for x in methods}
        clazz.methods = methods
        clazz.methods_list = list(methods.values())

    def gen_environment(self, d_num_classes, d_num_fields, d_num_methods, d_num_params, d_returns):
        clazzes = self.gen_cls_graph(d_num_classes(self.random))
        for i, clazz in enumerate(clazzes):
            if i == 0:
                continue
            self.gen_fields(clazz, clazzes, d_num_fields(self.random))
            self.gen_constructor(clazz, clazzes, d_num_params(self.random))
            self.gen_methods(clazz, clazzes, d_num_methods(self.random), d_num_params, d_returns)

        return MJEnv(clazzes)


class MJGenException(Exception):
    pass


class StopRunning(Exception):
    pass


def retry_with_mjgenexp(func):
    def call(self, *args, depth):
        if depth > self.max_depth:
            raise MJGenException()
        for i in range(self.retry_times):
            try:
                result = func(self, *args, depth=depth)
                assert isinstance(result, str) or isinstance(result, tuple)
                if isinstance(result, tuple):
                    x, y = result
                    assert (isinstance(x, str) and isinstance(y, dict)) or (isinstance(x, str) and isinstance(y, bool))
                return result
            except MJGenException:
                pass
            except AssertionError as e:
                traceback.print_exc()
                print("name:", func.__name__)
                print("args:", repr([self] + list(args) + list(depth)))
                print("res:", repr(result))
                raise StopRunning()
        raise MJGenException()

    return call


class MJMethodBodyGenerator:
    def __init__(self, env, seed=0):
        self.env = env
        self.random = random.Random(seed)

        self.d_gen_p_dist = CustomListDist([0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 3, 4])
        self.d_gen_exp_dist = CustomListDist([0, 0, 0, 0, 0, 0, 0, 1])
        self.d_gen_statement_dist = CustomListDist([0, 1, 1, 1, 1, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6])
        self.d_gen_pe_dist = CustomListDist([0, 1])
        self.d_num_statements = CustomListDist([10, 10, 10, 10, 5, 5, 5, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2])
        self.retry_times = 3
        self.max_depth = 6

    def get_random_clazz(self):
        return self.get_random_sample(self.env.clazzes_list)

    def get_random_sample(self, values):
        if not isinstance(values, list):
            values = list(values)
        if len(values) == 0:
            raise MJGenException()
        return self.random.sample(values, 1)[0]

    @retry_with_mjgenexp
    def gen_p(self, symbols, type, sur, *, depth):
        def gen_var_p():
            can_vars = [k for k, v in symbols.items() if type in v.idx_ancestors]
            return self.get_random_sample(can_vars)

        def gen_null_p():
            return "((" + type.name + ") null)" if sur else "null"

        def gen_field_p():
            field = self.get_random_sample(type.idx_field)
            return self.gen_p(symbols, field.cls, True, depth=depth + 1) + "." + field.name

        gen_funcs = {
            0: gen_var_p,
            1: gen_null_p,
            2: gen_field_p,
            3: lambda: self.gen_pe(symbols, type, depth=depth + 1),
            4: lambda: "(" + self.gen_exp(symbols, type, sur, depth=depth + 1) + ")",
        }

        return gen_funcs[self.d_gen_p_dist(self.random)]()

    @retry_with_mjgenexp
    def gen_pe(self, symbols, type, *, depth):
        if self.d_gen_exp_dist(self.random) == 0:
            method = self.get_random_sample(type.idx_method)
            exp = self.gen_p(symbols, method.cls, True, depth=depth + 1)
            params = [self.gen_exp(symbols, t, False, depth=depth + 1) for t, _ in method.params]
            return exp + "." + method.name + "(" + ", ".join(params) + ")"
        else:
            avilt = list(type.idx_descendants) + [type]
            t = self.get_random_sample(avilt)
            method = t.constructor
            params = [self.gen_exp(symbols, t, False, depth=depth + 1) for t, _ in method.params]
            return "new " + t.name + "(" + ", ".join(params) + ")"

    @retry_with_mjgenexp
    def gen_exp(self, symbols, type, sur, *, depth):
        if self.d_gen_exp_dist(self.random) == 0:
            return self.gen_p(symbols, type, sur, depth=depth + 1)
        else:
            t = self.get_random_sample(list(type.idx_descendants) + [type])
            return "(" + t.name + ") " + self.gen_exp(symbols, self.get_random_sample(t.idx_transferable), False, depth=depth + 1)

    @retry_with_mjgenexp
    def gen_statements(self, symbols, *, depth):
        num_statements = self.d_num_statements(self.random)
        result = []
        for _ in range(num_statements):
            new_stmt, symbols = self.gen_statement(symbols, depth=depth + 1)
            result.append(new_stmt)
        return "".join(result)

    @retry_with_mjgenexp
    def gen_statement(self, symbols, *, depth):
        def gen_statement_if():
            t1 = self.get_random_clazz()
            t2 = self.get_random_sample(t1.idx_transferable)
            return (
                "if("
                + self.gen_exp(symbols, t1, False, depth=depth + 1)
                + " == "
                + self.gen_exp(symbols, t2, False, depth=depth + 1)
                + ") {\n"
                + self.gen_statements(symbols, depth=depth + 1)
                + "} else {\n"
                + self.gen_statements(symbols, depth=depth + 1)
                + "}\n"
            )

        def gen_statement_field():
            t1 = self.get_random_clazz()
            f = self.get_random_sample(t1.fields_list)
            return self.gen_p(symbols, t1, True, depth=depth + 1) + "." + f.name + " = " + self.gen_exp(symbols, f.t, False, depth=depth + 1) + ";\n"

        def gen_statement_decl():
            nonlocal symbols

            t = self.get_random_clazz()
            name = f"VAR_{self.random.randint(0, 999):03d}"
            if name in symbols:
                raise MJGenException()
            symbols = dict(symbols)
            symbols[name] = t
            return t.name + " " + name + ";\n"

        def gen_statement_assign():
            var = self.get_random_sample(symbols.keys())
            t = symbols[var]
            return var + " = " + self.gen_exp(symbols, t, False, depth=depth + 1) + ";\n"

        gen_funcs = {
            0: lambda: ";\n",
            1: lambda: self.gen_pe(symbols, self.get_random_clazz(), depth=depth + 1) + ";\n",
            2: gen_statement_if,
            3: gen_statement_field,
            4: gen_statement_decl,
            5: gen_statement_assign,
            6: lambda: "{\n" + self.gen_statements(symbols, depth=depth + 1) + "}\n",
        }

        new_dist = CustomListDist(self.d_gen_statement_dist.values + [4] * (5 * min(1, 10 - len(symbols))))
        return gen_funcs[new_dist(self.random)](), symbols


def gen_random_env_body(random_seed):
    env_gen = MJEnvGenerator(random_seed)
    d_num_classes = CustomListDist([2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 6, 7, 8])
    d_num_fields = CustomListDist([0, 1, 2, 2, 2, 2, 2, 3])
    d_num_methods = CustomListDist([0, 1, 2,2, 2, 2, 2, 3])
    d_num_params = CustomListDist([0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 3, 4])
    d_returns = CustomListDist([True, True, True, False])
    mjenv = env_gen.gen_environment(d_num_classes, d_num_fields, d_num_methods, d_num_params, d_returns)
    envstr = mjenv.dump()

    body_gen = MJMethodBodyGenerator(mjenv, random_seed)
    while True:
        try:
            body = body_gen.gen_statements({}, depth=0)
            break
        except MJGenException:
            pass

    return envstr.encode("utf-8"), ("{\n" + body + "}\n").encode("utf-8")


def gen(random_seed, length):
    r = random.Random(random_seed)
    r_set = set()
    result = []

    for _ in range(length):
        seed = r.randint(0, 65536)
        while seed in r_set:
            seed = r.randint(0, 65536)
        r_set.add(seed)
        result.append(gen_random_env_body(seed))

    return result


def main():
    results = gen(0, 1000)
    for e, b in results:
        folder_name = hashlib.sha1(e + b"\n" + b).hexdigest()
        os.makedirs(f"right/{folder_name}", exist_ok=True)
        with open(f"right/{folder_name}/env", "wb") as fout:
            fout.write(e)
            fout.write(b"class CLSFIX extends Object\n{\nCLSFIX() {super();}\nvoid METHODFIX() {}\n}\n")
        with open(f"right/{folder_name}/block", "wb") as fout:
            fout.write(b)
        with open(f"right/{folder_name}/method", "wb") as fout:
            fout.write(b"CLSFIX.METHODFIX")


if __name__ == "__main__":
    main()
