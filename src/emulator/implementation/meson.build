file_list = files(
	'Call.cs',
	'Command.cs',
	'Emulator.cs',
	'Helpers.cs',
	'PortExtender.cs',
	'Statics.cs'
)

comp = find_program('csc')

engine = custom_target(
	'engine',
	output: 'engine.dll',
	input: file_list,
	command: [comp, '-t:library', '@INPUT@', '-out:@OUTPUT@'],
	install: true,
	install_dir: 'subdir',
)

pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()

conf = configuration_data()
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
    input: 'config.h.in',
    output: 'config.h',
    configuration: conf
)

engine_files = files(
	'emulator_shim.c',
)

shim_inc_dirs = [
  './',
]

engine_shim = library(
	'engine',
	engine_files,
	install: true,
	include_directories: shim_inc_dirs,
	dependencies: [dependency('mono-2')],
)

experiments = executable(
	'shim_experiments',
	'shim_experiments.c',
	install: false,
	dependencies: [dependency('mono-2')],
	link_with: engine_shim,
)
