export YAKMAN_ADAPTER=REDIS

redis-cli FLUSHALL
redis-cli SET CONFIG_MAN_CONFIGS '{"configs":[{"name":"testing-1","description":"This is a test desc for config 1"},{"name":"Testing-2","description":"This is a test desc for config 2"},{"name":"Testing-3","description":"This is a test desc for config 3"}]}'
redis-cli SET CONFIG_MAN_LABELS '{"labels":[{"name":"env","description":"My env label","priority":1,"options":["dev","prod"]},{"name":"my-label","description":"Here is my label","priority":2,"options":["option 1","option 2","option 3"]}]}'
redis-cli SET CONFIG_MAN_INSTANCE_META_100 '{"instances":[{"config_name":"testing-1","instance":"testing-1/ross.json","labels":[]},{"config_name":"testing-1","instance":"testing-1/ross_with_labels.json","labels":[{"label_type":"env","value":"dev"}]},{"config_name":"testing-1","instance":"testing-1/ross_prod_foo.json","labels":[{"label_type":"env","value":"prod"},{"label_type":"my-label","value":"foo"}]},{"config_name":"testing-1","instance":"testing-1/ross_prod_bar.json","labels":[{"label_type":"env","value":"prod"},{"label_type":"my-label","value":"bar"}]}]}'
redis-cli SET CONFIG_MAN_INSTANCE_testing-1/ross_with_labels.json '{"data":"test data","labels":"some labels"}'
redis-cli SET CONFIG_MAN_INSTANCE_testing-1/ross.json '{"data":"test data","labels":"No labels!"}'
redis-cli SET CONFIG_MAN_INSTANCE_testing-1/ross_prod_foo.json '{"data":"test data","labels":"Foo!"}'
redis-cli SET CONFIG_MAN_INSTANCE_testing-1/ross_prod_bar.json '{"data":"test data","labels":"Bar!"}'

