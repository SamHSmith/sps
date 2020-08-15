import std.stdio;

void main(string[] args)
{
    writeln("Welcome to the SPS package manager");
    if (args.length <= 1)
    {
        args ~= "";
    }

    switch (args[1])
    {
    case "repo":
        goto case "repository";

    case "repository":
        repository_cli(args);
        break;

    default:
        writeln("Error, unrecognised argument ", args[1]);
        break;
    }
}

void repository_cli(string[] args)
{
    writeln("Welcome to the repository CLI");
}
