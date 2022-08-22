import os
import yaml

with open(os.path.dirname(__file__) + "../msg_ids.yaml", 'r') as stream:
    try:
        print('>> Reading yaml ...')
        data = yaml.safe_load(stream)
    except yaml.YAMLError as exc:
        print(exc)

with open(os.path.dirname(__file__) + '/msg.hpp', 'w') as fout:
    print('>> Generating msg.hpp ...')
    fout.write('#pragma once\n\n')
    for x,y in data.items():
        # TODO: #define for len(y) == 1
        if type(y) is int:
            fout.write('#define ' + x + ' ' + str(y) + '\n')

    fout.write('\n')
    fout.write('namespace dh {\n\n')

    for x,y in data.items():
        if type(y) is list:
            fout.write('enum ' + x + ' : uint16_t  { \n')
            for yi in y[:-1]:
                k = list(yi)[0];
                fout.write('    ' + k + '= ' + str(yi[k]) + ',\n')
            k = list(y[-1])[0];
            fout.write('    ' + k + '= ' + str(y[-1][k]) + '\n')
            fout.write('};\n')
    fout.write('}\n')
    print('>> Success!')
