# Scheduler benchmark report

Bootstrap: 10000 resamples; seed 20260913; 95% median CI.
Censored failure durations are +inf; null means unavailable, not zero.

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 13185033.244444445 | 12900075.377777778 | 2085485.0188951292 | [11067033.6, 15971209.955555556] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.4590314296608636 | 0.47192982456140353 | 0.08905578153294849 | [0.3414190050375286, 0.5500132310134956] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.07932609118316843 | 0.08914906534222038 | 0.043184096909620405 | [0.014139093075790346, 0.12870641850376643] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 56.843986849934275 | 55.67581593046058 | 13.552459884916882 | [40.51735046448467, 72.12612259731661] | 0.0 |
| adaptive | switch_count | 5 | 0 | 380.4 | 339.0 | 335.98333291995306 | [49.0, 840.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 8.453299852595867 | 7.533165929646008 | 7.466301444370871 | [1.0888888888888888, 18.666666666666668] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 9.088848988540995 | 8.97757827603831 | 0.7979982549147272 | [8.088888888888889, 9.977777777777778] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.6347566668109204 | 0.6395167774955647 | 0.035799511655697426 | [0.5879747549412512, 0.6733644140920249] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 15.846766127098505 | 15.529127454545453 | 2.9794273516388454 | [12.555323829787236, 19.55713823529413] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1861.2 | 1869.0 | 47.594117283546716 | [1804.0, 1931.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 308.9628 | 395.873 | 154.53679704102836 | [46.391000000000005, 408.548] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 1393.2 | 1234.0 | 329.4369438906329 | [1158.0, 1961.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 1393.2 | 1234.0 | 329.4369438906329 | [1158.0, 1961.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 45339.2 | 46537.0 | 8897.169223972309 | [33752.0, 54041.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.21251310336240814 | 0.20309097668490095 | 0.062074992240846234 | [0.13608657434759427, 0.28732340221047575] | 0.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 13736419.697777778 | 14518345.955555556 | 2255994.690514772 | [10139633.777777778, 16189256.533333331] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.4307435299442606 | 0.4011679869997969 | 0.09322216483725371 | [0.3256180689795503, 0.5765234263426934] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.06280532351144033 | 0.05361676793212248 | 0.030105902531968036 | [0.0414157965503588, 0.11537027043806893] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 53.31750986089452 | 49.217143785657335 | 15.957343013191418 | [37.33614593109365, 80.12562014074057] | 0.0 |
| classic | switch_count | 5 | 0 | 7276.4 | 8592.0 | 4986.321219095295 | [12.0, 13534.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 161.69639321595324 | 190.92909046465635 | 110.80710685208749 | [0.26666666666666666, 300.75555555555553] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 8.79992217456896 | 8.777777777777779 | 0.929228037911856 | [7.555555555555555, 10.155555555555557] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.6234694214292181 | 0.6091951209062922 | 0.03950722655531919 | [0.5826616355766562, 0.6872543553059254] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 16.870470496799832 | 18.188778709677425 | 3.3323302414840756 | [11.315459534883724, 19.990280144927535] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1810.0 | 1785.0 | 124.73572062564917 | [1690.0, 2021.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 358.4555 | 385.134 | 73.09062598069606 | [229.04, 408.671] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 1433.4 | 1529.0 | 613.0989316578524 | [422.0, 1922.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 1433.4 | 1529.0 | 613.0989316578524 | [422.0, 1922.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 42765.2 | 39499.0 | 9900.15127157156 | [32005.0, 58535.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.19152096942785415 | 0.18780342656685564 | 0.029376394741015045 | [0.15063318004952247, 0.2317744004522427] | 0.0 |
| classic | load[0].reached_ms | 5 | 0 | +inf | 1000.0 | None | [1000.0, +inf] | 0.2 |
| edpf | useful_goodput_bps | 5 | 0 | 16499341.226666668 | 15795509.333333334 | 4739857.706779555 | [11059547.022222225, 24024428.088888887] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.3184425445286853 | 0.348887264863999 | 0.2009342001388053 | [0.0, 0.549970771432603] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.05161798036091716 | 0.06152248994096462 | 0.03155404869186135 | [0.002720673139384918, 0.0874521847751057] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 41.14220634948615 | 41.08059292013685 | 21.021003063287846 | [13.393774727424105, 72.3356931701217] | 0.0 |
| edpf | switch_count | 5 | 0 | 5047.0 | 4368.0 | 4302.224308424655 | [254.0, 11936.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 112.15555555555554 | 97.06666666666666 | 95.60498463165901 | [5.644444444444445, 265.24444444444447] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 7.528888888888889 | 7.844444444444444 | 2.17144203096172 | [4.022222222222222, 10.0] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.5876646727908577 | 0.5947965499506376 | 0.06934648771822607 | [0.4834392003163496, 0.6755728502679864] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 18.86274712531157 | 18.88520191176471 | 4.499978890676981 | [12.321574680851066, 24.878366990291266] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1871.0 | 1873.0 | 91.45490692138941 | [1732.0, 1988.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 232.1076 | 301.6975 | 176.27858149864662 | [40.985, 403.113] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 977.8 | 1131.0 | 435.28978853173203 | [293.0, 1413.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 977.8 | 1131.0 | 435.28978853173203 | [293.0, 1413.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 31571.6 | 34850.0 | 19914.292864673855 | [0.0, 54567.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.13941472469181154 | 0.14379294749281474 | 0.08968126231083304 | [0.005000096901102735, 0.2523145290704851] | 0.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 16967813.83111111 | 17677915.733333334 | 4694426.306345821 | [10167240.533333331, 22816515.555555556] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.29277091040292363 | 0.26760506817132185 | 0.1990709446189248 | [0.03939906681023432, 0.5761481481481482] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.04758467931707548 | 0.023888304705668273 | 0.04276015091981408 | [0.0034138954968337596, 0.09590794873865527] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 39.48581290250668 | 30.97398832618538 | 23.535883892973732 | [17.297411855272088, 77.63496208686136] | 0.0 |
| enhanced | switch_count | 5 | 0 | 1003.8 | 1111.0 | 712.9324652447804 | [13.0, 1910.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 22.30666538274458 | 24.68888888888889 | 15.842945902524475 | [0.28888246927846045, 42.44444444444444] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 7.297733926900389 | 6.844444444444444 | 1.929655472198891 | [4.933333333333334, 9.866447412279728] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.5798720499205731 | 0.5670694140197152 | 0.07068082032940558 | [0.4989039228026012, 0.6880215653921629] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.330293813339114 | 20.70552565789475 | 4.877629374471184 | [11.290701395348837, 23.949084845360822] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1902.0 | 1879.0 | 63.470465572579506 | [1857.0, 2012.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 169.5351 | 44.2985 | 175.73791517711823 | [42.621, 404.342] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 1127.6 | 1085.0 | 354.84334571751515 | [647.0, 1635.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 1127.6 | 1085.0 | 354.84334571751515 | [647.0, 1635.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 29252.0 | 26654.0 | 20205.91538386717 | [3842.0, 58335.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.12278068624665912 | 0.09148647402195254 | 0.08281173674785103 | [0.032171974588954357, 0.2364079571671092] | 0.0 |
| enhanced | load[0].reached_ms | 5 | 0 | +inf | 1000.0 | None | [1000.0, +inf] | 0.2 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 19042063.786666665 | 19375497.244444445 | 3619626.180192764 | [13918483.91111111, 24019281.066666663] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.20755215829265108 | 0.1959639088052848 | 0.1526019417596811 | [0.0, 0.4263829524905474] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.03080844700380909 | 0.022009225563486612 | 0.02928964518736316 | [0.004559088523037806, 0.07849043076140565] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 29.31760172065882 | 27.06740569431419 | 13.416673929679312 | [13.17460100350788, 50.19703806309837] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 1312.0 | 1355.0 | 628.1289676491604 | [453.0, 2203.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 29.155443953097336 | 30.11111111111111 | 13.958461926420414 | [10.066666666666666, 48.95555555555555] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 6.49774666735801 | 6.555555555555555 | 1.716114029932743 | [3.955555555555556, 8.733333333333333] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.5486398084514755 | 0.542180905354335 | 0.04944117766473555 | [0.4833000485507355, 0.6211634961407299] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 21.450780225744342 | 21.84402373493975 | 2.7930438273256506 | [17.16482220338983, 24.883521359223295] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1903.0 | 1880.0 | 63.17436188834834 | [1850.0, 2004.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 100.11349999999999 | 43.584 | 126.9759353844263 | [41.946, 327.25] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 915.8 | 1162.0 | 414.7971793539585 | [315.0, 1234.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 915.8 | 1162.0 | 414.7971793539585 | [315.0, 1234.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 20298.0 | 19460.0 | 14837.042343405237 | [0.0, 41499.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.09103123305441693 | 0.08375310308259623 | 0.06878582613433024 | [0.00546342219466832, 0.1961174383713798] | 0.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | load[0] graded=True | 0.0 | 0.8 | 0.19999999999999996 |
| classic | settled_rate | — | 0.8 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | load[0] graded=True | 0.2 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | load[0] graded=True | 0.2 | 0.8 | 0.19999999999999996 |
| enhanced | settled_rate | — | 0.8 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | load[0] graded=True | 0.2 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.7049914737420158, 1.4372865712967238] | 7.076183756160664 | None | () |
| adaptive | edpf | 5 | [0.47507011530071674, 1.3177356574716532] | 12.304255969740453 | None | () |
| adaptive | enhanced | 5 | [0.5830570312211115, 1.2687882553269827] | 20.432475639008167 | None | () |
| adaptive | rtt-threshold | 5 | [0.4607562386768746, 0.824299358827294] | 27.596591575611875 | None | () |
| classic | adaptive | 5 | [0.6957554742181981, 1.4184568711052803] | -7.076183756160664 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.6738664693050794, 0.9191438939494927] | 5.228072213579788 | None | () |
| classic | enhanced | 5 | [0.6364111766213791, 1.309517235031525] | 13.356291882847504 | None | () |
| classic | rtt-threshold | 5 | [0.5584547785637893, 1.16314798628387] | 20.520407819451208 | None | () |
| edpf | adaptive | 5 | [0.7588775444679895, 2.1049524434240734] | -12.304255969740453 | None | () |
| edpf | classic | 5 | [1.087968931287869, 1.4839735252463946] | -5.228072213579788 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.7277765803491704, 1.4517695245984632] | 8.128219669267716 | None | () |
| edpf | rtt-threshold | 5 | [0.609120311312124, 1.7260808175889195] | 15.292335605871422 | None | () |
| enhanced | adaptive | 5 | [0.7881535755091676, 1.7150980889565366] | -20.432475639008167 | None | () |
| enhanced | classic | 5 | [0.7636401975083026, 1.5713111848677215] | -13.356291882847504 | None | () |
| enhanced | edpf | 5 | [0.688814569431456, 1.374048062277881] | -8.128219669267716 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.5150455692902093, 1.2701035433335575] | 7.164115936603705 | None | () |
| rtt-threshold | adaptive | 5 | [1.2131514956200744, 2.170345002536783] | -27.596591575611875 | None | () |
| rtt-threshold | classic | 5 | [0.8597358305153329, 1.790655283802492] | -20.520407819451208 | None | () |
| rtt-threshold | edpf | 5 | [0.5793471486444376, 1.6417117955660854] | -15.292335605871422 | None | () |
| rtt-threshold | enhanced | 5 | [0.787337383041516, 1.9415757743108293] | -7.164115936603705 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@baseline--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| upstream-classic | useful_goodput_bps | 5 | 0 | 24012543.146666665 | 24011326.577777777 | 9427.276777908322 | [23999394.844444446, 24023492.266666666] | 0.0 |
| upstream-classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-classic | per_link_share_gini | 5 | 0 | 0.004356052288775966 | 0.0045560129972141246 | 0.000422358754528387 | [0.0038472311661486014, 0.004712142330668116] | 0.0 |
| upstream-classic | cpu_ms_per_mb | 5 | 0 | 12.20095104957522 | 12.212292090908125 | 0.46963648935636776 | [11.551230090492336, 12.876285008842899] | 0.0 |
| upstream-classic | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | sender_cpu_percent | 5 | 0 | 3.662222222222222 | 3.6666666666666665 | 0.14177011473543608 | [3.466666666666667, 3.8666666666666663] | 0.0 |
| upstream-classic | diagnostics.loss_ratio | 5 | 0 | 0.48408697980002147 | 0.4841500861272686 | 0.0001718438939994132 | [0.4839028826406967, 0.4842527161690772] | 0.0 |
| upstream-classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 24.87683145631068 | 24.87822233009709 | 0.006008915815753726 | [24.86687475728155, 24.88227378640776] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 2001.0 | 2001.0 | 2.7386127875258306 | [1998.0, 2005.0] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-classic | diagnostics.ms_rtt_median | 5 | 0 | 46.37839999999999 | 46.367 | 0.32116319838985363 | [45.949, 46.74] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_delta | 5 | 0 | 294.2 | 293.0 | 18.376615575235828 | [267.0, 316.0] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_sum | 5 | 0 | 294.2 | 293.0 | 18.376615575235828 | [267.0, 316.0] | 0.0 |
| upstream-classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | diagnostics.retrans_ratio | 5 | 0 | 0.004961967048531553 | 0.005053927928276823 | 0.0002641531050196669 | [0.004496124031007752, 0.005121552149793298] | 0.0 |
| upstream-classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| upstream-enhanced | useful_goodput_bps | 5 | 0 | 23828045.795555554 | 23996119.466666665 | 308867.2614197915 | [23292381.155555554, 24018813.155555554] | 0.0 |
| upstream-enhanced | viewer_loss_ratio | 5 | 0 | 0.007336740496247975 | 0.0 | 0.013458481750314602 | [0.0, 0.0310070538796338] | 0.0 |
| upstream-enhanced | per_link_share_gini | 5 | 0 | 0.012799566452649614 | 0.014812889106269767 | 0.009944066623236075 | [0.0005906359116726693, 0.026678318024658476] | 0.0 |
| upstream-enhanced | cpu_ms_per_mb | 5 | 0 | 13.509153950685398 | 12.888208773338508 | 2.076495108045386 | [11.631510315608663, 16.56240189448233] | 0.0 |
| upstream-enhanced | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | sender_cpu_percent | 5 | 0 | 4.017777777777778 | 3.8666666666666663 | 0.5651177087044414 | [3.488888888888889, 4.822222222222222] | 0.0 |
| upstream-enhanced | diagnostics.loss_ratio | 5 | 0 | 0.47968692214520753 | 0.4832399866338831 | 0.005674285453257021 | [0.4725069534942524, 0.48424459461484287] | 0.0 |
| upstream-enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 25.35297732412647 | 24.94895631067961 | 0.6530858408711255 | [24.88913980582525, 26.35526969696969] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1872.8 | 1996.0 | 170.07263154311454 | [1685.0, 1998.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rtt_median | 5 | 0 | 47.1918 | 46.435 | 1.6911317807906023 | [45.744, 50.035] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 1616.4 | 600.0 | 1733.1465027515708 | [345.0, 4245.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 1616.4 | 600.0 | 1733.1465027515708 | [345.0, 4245.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 734.6 | 0.0 | 1344.9062420852988 | [0.0, 3099.0] | 0.0 |
| upstream-enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.051864838350001205 | 0.008705387302874129 | 0.07098025387989422 | [0.005591564283641289, 0.16880530973451327] | 0.0 |
| upstream-enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| upstream-classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-classic | settled_rate | — | 1.0 | — |
| upstream-classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| upstream-enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-enhanced | settled_rate | — | 1.0 | — |
| upstream-enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| upstream-classic | upstream-classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| upstream-classic | upstream-enhanced | 5 | [0.9995908984648952, 1.0312176699243665] | 0.0 | None | () |
| upstream-enhanced | upstream-classic | 5 | [0.9697273710150291, 1.0004092689676678] | 0.0 | None | () |
| upstream-enhanced | upstream-enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 9596248.604444444 | 9596856.888888888 | 1557.1671225353011 | [9594049.422222223, 9597558.755555555] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.09001050560123355 | 0.0897039226989554 | 0.002977230843037982 | [0.08626334469587567, 0.0945220734562258] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 16.747285259898767 | 16.856138305392772 | 0.446142076483802 | [16.116387735836476, 17.22660288353327] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2210.2 | 2208.0 | 6.016643582596529 | [2204.0, 2218.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 49.11533788138041 | 49.06666666666667 | 0.13398416615569086 | [48.97668940690207, 49.28888888888889] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 2.008879901434289 | 2.022177284949223 | 0.05351540295014053 | [1.9333333333333331, 2.066666666666667] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.34406745736802313 | 0.34605533194393623 | 0.004995694330761135 | [0.33557968906558067, 0.34821187637899015] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 9.942720780487807 | 9.944848048780488 | 0.006130656897722883 | [9.932917560975614, 9.948789512195122] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1961.8 | 1963.0 | 2.16794833886788 | [1959.0, 1964.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 66.0862 | 65.989 | 2.3578099372086814 | [62.882, 69.045] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 64.8 | 63.0 | 7.362064927722384 | [57.0, 73.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 64.8 | 63.0 | 7.362064927722384 | [57.0, 73.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.0037709360486418105 | 0.003843066669909761 | 0.0003797239063151627 | [0.003360771516243729, 0.004230488694383662] | 0.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 4632694.3288888885 | 4649164.8 | 172045.76807457086 | [4366546.488888889, 4832585.955555555] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.5108519482643152 | 0.5068212324588164 | 0.01855987666062194 | [0.49133034379671153, 0.5396845457762499] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.05471267097791066 | 0.03882607895591076 | 0.03546585914288489 | [0.021482661544995525, 0.1122495242384734] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 60.89137569697673 | 59.87248628263398 | 2.0080737117626333 | [58.85967617760554, 63.37349804809625] | 0.0 |
| classic | switch_count | 5 | 0 | 15.4 | 15.0 | 1.816590212458495 | [13.0, 18.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 0.34222222222222226 | 0.3333333333333333 | 0.04036867138796658 | [0.28888888888888886, 0.4] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 3.5244444444444447 | 3.466666666666667 | 0.12726952056245647 | [3.422222222222222, 3.7333333333333334] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.630484124075647 | 0.6327407288231035 | 0.010703906028316706 | [0.6140847821840203, 0.6422822964095499] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 5.580273402339181 | 5.520579999999999 | 0.18276911918884703 | [5.409162222222221, 5.867082000000001] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1635.4 | 1633.0 | 62.448378681916154 | [1542.0, 1706.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 1061.912 | 1055.265 | 24.99116408853337 | [1038.83, 1090.7] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 572.4 | 448.0 | 414.47593416264834 | [254.0, 1270.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 572.4 | 448.0 | 414.47593416264834 | [254.0, 1270.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 20473.6 | 20455.0 | 628.8169845034404 | [19722.0, 21351.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.28611722832850345 | 0.27695273141455296 | 0.018101371318262904 | [0.2689125565662568, 0.31030118957226016] | 0.0 |
| classic | load[0].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | useful_goodput_bps | 5 | 0 | 4638636.800000001 | 4651504.355555556 | 34373.955330176745 | [4589974.044444445, 4673028.266666667] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.5099656362490335 | 0.5103022143149087 | 0.0022448302933553354 | [0.506511905353681, 0.512337496813663] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.08268994787195526 | 0.08482005716047518 | 0.02749435598788326 | [0.04628162394256038, 0.11570686312097962] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 61.250052575714186 | 61.53325899402218 | 1.0888379078500372 | [60.10853623388216, 62.74545285252448] | 0.0 |
| edpf | switch_count | 5 | 0 | 15.8 | 16.0 | 0.8366600265340756 | [15.0, 17.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 0.351109432136076 | 0.35555555555555557 | 0.01858943503099158 | [0.3333333333333333, 0.37776938290260215] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 3.551095210229896 | 3.5555555555555554 | 0.03973899997512144 | [3.511111111111111, 3.6] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.6289015056826143 | 0.6289187152226552 | 0.0015986064175750841 | [0.6269014576518784, 0.6308396946564886] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 5.591343494736842 | 5.6113975 | 0.06461275947301127 | [5.480494, 5.6363224999999995] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1670.6 | 1660.0 | 29.211299183706295 | [1649.0, 1722.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 1059.9175 | 1062.83 | 24.76111972427743 | [1027.6825, 1091.32] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 506.0 | 525.0 | 136.4532887108259 | [345.0, 662.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 506.0 | 525.0 | 136.4532887108259 | [345.0, 662.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 20752.4 | 20867.0 | 499.0218432092928 | [20099.0, 21225.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.2886980712177843 | 0.2876402461093015 | 0.006264220042388526 | [0.28161595054634103, 0.2987770623568558] | 0.0 |
| edpf | load[0].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 9597044.053333335 | 9596856.888888888 | 1197.5233992507972 | [9595687.111111112, 9598962.48888889] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.08890693960359813 | 0.08933099991970699 | 0.008541838783543476 | [0.08028685376115186, 0.09966636310707656] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 16.671787659839502 | 16.668468095922368 | 0.6545691799688597 | [15.93308401144662, 17.413108588145157] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2213.4 | 2217.0 | 5.683308895353129 | [2205.0, 2218.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 49.1864483999121 | 49.266666666666666 | 0.12645981041652246 | [49.0, 49.28888888888889] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 1.9999908150189256 | 2.0 | 0.078557680068935 | [1.9111111111111112, 2.088888888888889] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.3462632901295465 | 0.34521324900027084 | 0.004171121957919241 | [0.3406905174071937, 0.35152872874136243] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 9.944533951219514 | 9.944759512195125 | 0.0027646494582512686 | [9.941835853658535, 9.948777804878048] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1964.2 | 1965.0 | 2.7748873851023212 | [1960.0, 1967.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 65.476 | 65.589 | 1.8033276185984635 | [63.244, 67.718] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 70.0 | 68.0 | 7.0710678118654755 | [61.0, 79.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 70.0 | 68.0 | 7.0710678118654755 | [61.0, 79.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.003984156656321949 | 0.003965261391003966 | 0.00031490771893325864 | [0.0035767293608116985, 0.004424671188583376] | 0.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 4600408.462222222 | 4606584.888888889 | 82059.49005673254 | [4477675.377777778, 4708589.511111111] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.5145141619943491 | 0.5138491844235712 | 0.009696366815779156 | [0.5013217616838745, 0.5284893347618583] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.06772701111539568 | 0.0723236095336448 | 0.04024850111172289 | [0.015565300048591987, 0.12551767180424453] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 60.442273958943204 | 60.706753436852914 | 1.0271620000051347 | [59.045910704058805, 61.53986886211273] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 15.6 | 15.0 | 0.8944271909999159 | [15.0, 17.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 0.3466666666666667 | 0.3333333333333333 | 0.019876159799998138 | [0.3333333333333333, 0.37777777777777777] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 3.475555555555556 | 3.4444444444444446 | 0.07633584016585633 | [3.4, 3.6] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.6299281146250728 | 0.6300701582639908 | 0.0058048079899260495 | [0.6212830017644108, 0.6364453469598362] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 5.554182342105263 | 5.509537368421052 | 0.09196970865941266 | [5.494896842105263, 5.713225500000001] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1628.8 | 1605.0 | 50.70207096362041 | [1570.0, 1686.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 999.8976 | 1021.77 | 87.4555557343271 | [874.75, 1079.53] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 650.4 | 584.0 | 204.3925145400389 | [472.0, 999.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 650.4 | 584.0 | 204.3925145400389 | [472.0, 999.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 20990.2 | 21364.0 | 699.4670828566559 | [20102.0, 21704.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.28376637259076826 | 0.2865963921845367 | 0.012319046173623893 | [0.271118993036031, 0.30030774491366047] | 0.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | load[0] graded=True | 0.0 | 0.0 | 1.0 |
| classic | settled_rate | — | 0.2 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | load[0] graded=True | 0.0 | 0.0 | 1.0 |
| edpf | settled_rate | — | 0.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | load[0] graded=True | 0.0 | 0.0 | 1.0 |
| rtt-threshold | settled_rate | — | 0.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [1.9860089078233927, 2.1979747106729532] | -50.68212324588164 | None | () |
| adaptive | edpf | 5 | [2.0538199659557423, 2.0909832305418217] | -51.03022143149087 | None | () |
| adaptive | enhanced | 5 | [0.9996830891494601, 1.000195050591247] | 0.0 | None | () |
| adaptive | rtt-threshold | 5 | [2.0383086554705354, 2.1426406813313132] | -51.38491844235712 | None | () |
| classic | adaptive | 5 | [0.45496428832606095, 0.5035224142554177] | 50.68212324588164 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.9344147391609091, 1.0528569244100106] | -0.3480981856092358 | None | () |
| classic | enhanced | 5 | [0.45505302937949527, 0.5035715156390941] | 50.68212324588164 | None | () |
| classic | rtt-threshold | 5 | [0.9468824514230633, 1.052510580490099] | -0.7027951964754808 | None | () |
| edpf | adaptive | 5 | [0.4782439119518319, 0.48689759403261584] | 51.03022143149087 | None | () |
| edpf | classic | 5 | [0.9497966692486445, 1.0701885983711958] | 0.3480981856092358 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.4782905482825032, 0.48699256369620864] | 51.03022143149087 | None | () |
| edpf | rtt-threshold | 5 | [0.9748087051575077, 1.0310883536234914] | -0.35469701086624505 | None | () |
| enhanced | adaptive | 5 | [0.9998049874460669, 1.0003170113148654] | 0.0 | None | () |
| enhanced | classic | 5 | [1.9858152594887686, 2.1975460780111447] | -50.68212324588164 | None | () |
| enhanced | edpf | 5 | [2.0534194452788626, 2.090779346551812] | -51.03022143149087 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [2.038109907582232, 2.143319922670986] | -51.38491844235712 | None | () |
| rtt-threshold | adaptive | 5 | [0.46671381193913375, 0.49060283255734594] | 51.38491844235712 | None | () |
| rtt-threshold | classic | 5 | [0.9501092136616361, 1.0560972996142306] | 0.7027951964754808 | None | () |
| rtt-threshold | edpf | 5 | [0.969848991588122, 1.0258422957337274] | 0.35469701086624505 | None | () |
| rtt-threshold | enhanced | 5 | [0.46656590526803343, 0.4906506740778664] | 51.38491844235712 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B2@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 8556596.906666666 | 8446731.377777778 | 897871.2617985314 | [7685206.044444445, 9888365.51111111] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.32877876004769324 | 0.33658536585365856 | 0.0686900601128958 | [0.22567241699128177, 0.3940843274064071] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.09135781950232569 | 0.08576522355543334 | 0.030793980677185164 | [0.05401619032497987, 0.13050631058972015] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 43.528258677494605 | 45.8823109463576 | 8.26776422966927 | [32.00169371660767, 51.58540215470605] | 0.0 |
| adaptive | switch_count | 5 | 0 | 241.4 | 145.0 | 275.7123501042346 | [8.0, 643.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 5.364380939682822 | 3.2222222222222223 | 6.126825488438448 | [0.17777777777777778, 14.288571365080776] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 4.582204642365972 | 4.844444444444444 | 0.445695891018378 | [3.955467656274305, 4.955555555555556] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.5741567794889401 | 0.5764713615846444 | 0.022961199650218617 | [0.5396990728804645, 0.597073567267308] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 9.783909463580946 | 9.721248055555554 | 0.929783577197002 | [8.8522415625, 11.161080000000004] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1722.2 | 1706.0 | 135.96948187001377 | [1588.0, 1883.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 552.1060999999999 | 560.1535 | 17.86195723598056 | [525.888, 568.56] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 595.6 | 666.0 | 324.10384138420824 | [200.0, 948.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 595.6 | 666.0 | 324.10384138420824 | [200.0, 948.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 17709.8 | 17595.0 | 3794.6452666883106 | [12166.0, 21455.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.16505110514716573 | 0.18009208960629106 | 0.030741951309632053 | [0.13227584352735583, 0.19913693295442583] | 0.0 |
| adaptive | load[0].reached_ms | 5 | 0 | +inf | 1000.0 | None | [1000.0, +inf] | 0.4 |
| classic | useful_goodput_bps | 5 | 0 | 7659190.186666667 | 7637713.066666666 | 40331.18287059821 | [7625079.466666667, 7724510.577777778] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.3956804637292559 | 0.39616538364234344 | 0.0032589552215027307 | [0.39019711722903233, 0.3987060763668654] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.1151589574713403 | 0.1301027879463924 | 0.024240621139726106 | [0.07532591387806106, 0.1313156424466189] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 51.34615579815822 | 50.75280547140272 | 1.7751485496516806 | [49.811303609299124, 54.235463848222636] | 0.0 |
| classic | switch_count | 5 | 0 | 8.4 | 8.0 | 0.5477225575051662 | [8.0, 9.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 0.18666587656076777 | 0.17777777777777778 | 0.01217233376055508 | [0.17777382724828336, 0.2] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 4.91553254372125 | 4.866666666666666 | 0.15973659978048674 | [4.7555555555555555, 5.177662718606253] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.5950424700969974 | 0.5942236854972781 | 0.00342430198556715 | [0.590885704589988, 0.5994588500563698] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.900141801136362 | 8.921192424242422 | 0.10980053699877126 | [8.78729303030303, 9.03246575757576] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1885.2 | 1883.0 | 11.30044246921332 | [1876.0, 1904.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 563.7057 | 563.486 | 2.6644586129268304 | [559.717, 566.987] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 303.4 | 281.0 | 133.9171385596332 | [174.0, 496.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 303.4 | 281.0 | 133.9171385596332 | [174.0, 496.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 21686.8 | 21859.0 | 334.4715234515489 | [21292.0, 22001.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.1871441395857464 | 0.1896154670094259 | 0.011161966819700211 | [0.1733280792614276, 0.19817992580929253] | 0.0 |
| classic | load[0].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | useful_goodput_bps | 5 | 0 | 8699496.959999999 | 8383329.422222222 | 985368.5252104821 | [7637713.066666666, 10283048.533333331] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.321165483480395 | 0.3458622449944225 | 0.07434789241808379 | [0.20014968659369445, 0.3992043163115156] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.09577231387029753 | 0.10448669282996029 | 0.049668064146868494 | [0.018387688590853796, 0.1488333014172549] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 41.58712391178076 | 42.61199604390229 | 7.528215613928799 | [30.081870403568004, 50.74235601321126] | 0.0 |
| edpf | switch_count | 5 | 0 | 667.0 | 397.0 | 739.0402560077496 | [7.0, 1908.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 14.822222222222223 | 8.822222222222223 | 16.42311680017221 | [0.15555555555555556, 42.4] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 4.44888888888889 | 4.444444444444445 | 0.36528358677256795 | [3.8666666666666663, 4.844444444444444] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.5726587752064365 | 0.5820958799869551 | 0.02499451087010175 | [0.5316530563539581, 0.5974219245836343] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 9.799631556087188 | 9.426844444444445 | 0.9867766053707946 | [8.79369181818182, 11.392514090909092] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1770.4 | 1783.0 | 115.86759685088838 | [1613.0, 1912.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 505.1197 | 551.6885 | 109.9847667586289 | [308.705, 564.587] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 563.8 | 623.0 | 248.35398124451316 | [203.0, 856.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 563.8 | 623.0 | 248.35398124451316 | [203.0, 856.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 17237.8 | 18293.0 | 4131.920764003105 | [10697.0, 21975.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.14993716507904015 | 0.15590053940271018 | 0.030983837447754292 | [0.0967741935483871, 0.17436803479206306] | 0.0 |
| edpf | load[0].reached_ms | 5 | 0 | +inf | 1000.0 | None | [1000.0, +inf] | 0.2 |
| enhanced | useful_goodput_bps | 5 | 0 | 9135496.533333333 | 8980851.91111111 | 1011812.52732731 | [8078251.377777778, 10429504.711111112] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.2855303659705621 | 0.29856202323889813 | 0.07920488562940071 | [0.1832391149713601, 0.3661332785630263] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.11592133113423311 | 0.12136378092555669 | 0.04146598737858286 | [0.048800903035915666, 0.15934413357222277] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 37.98870860222497 | 38.4026918940944 | 7.629048968454051 | [27.443510516591868, 45.859720173416605] | 0.0 |
| enhanced | switch_count | 5 | 0 | 402.4 | 326.0 | 383.5372211402695 | [38.0, 902.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 8.942222222222222 | 7.2444444444444445 | 8.523049358672656 | [0.8444444444444444, 20.044444444444444] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 4.262222222222222 | 4.311111111111111 | 0.4395283667582618 | [3.577777777777778, 4.7555555555555555] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.5588256439049407 | 0.5626103598126394 | 0.02853926569489766 | [0.5229506170296051, 0.5908533648583563] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.317255443578386 | 10.266766153846154 | 1.064362820295415 | [9.07130114285714, 11.584318222222228] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1721.8 | 1707.0 | 97.43561977018466 | [1623.0, 1877.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 420.77290000000005 | 549.746 | 222.24728978280024 | [49.324, 565.8734999999999] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 672.4 | 731.0 | 230.41440927164254 | [329.0, 925.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 672.4 | 731.0 | 230.41440927164254 | [329.0, 925.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 15324.0 | 16008.0 | 4262.157786849286 | [9789.0, 19609.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.14831823355870496 | 0.1627384146783668 | 0.03410217200289474 | [0.09796808533609726, 0.18159496649444984] | 0.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 8595012.408888888 | 7776916.622222222 | 1209792.541060501 | [7671636.622222222, 10192273.777777778] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.3256557531566219 | 0.3883587253753 | 0.09179621121373818 | [0.20538902112466528, 0.3966230123312872] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.12410011649499875 | 0.13215778806073672 | 0.019172569094190215 | [0.09125632694692368, 0.1396139925062867] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 43.760723868651425 | 47.77671840969089 | 11.206898266801774 | [29.82651434097183, 55.15265281015743] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 156.0 | 9.0 | 207.12677277454983 | [8.0, 441.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 3.466665876560768 | 0.2 | 4.602817878469159 | [0.17777382724828336, 9.8] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 4.568865383238398 | 4.644444444444445 | 0.6227039117650504 | [3.8, 5.288771360636431] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.5742508816976208 | 0.592695483458456 | 0.029551545875803713 | [0.5356747106623985, 0.597885686713164] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 9.744907518847008 | 8.921910909090908 | 1.2237025630487819 | [8.80037606060606, 11.333595909090908] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1834.2 | 1891.0 | 95.72199329307765 | [1697.0, 1910.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 468.0263 | 561.395 | 170.56235186274256 | [169.8275, 565.454] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 362.6 | 342.0 | 151.1135334773163 | [183.0, 603.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 362.6 | 342.0 | 151.1135334773163 | [183.0, 603.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 17589.8 | 20877.0 | 5109.393770301913 | [11045.0, 21775.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.15958863323979908 | 0.1780844332933348 | 0.03381221464024318 | [0.11861672743343275, 0.19024548677835815] | 0.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | +inf | +inf | None | [1000.0, +inf] | 0.6 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | load[0] graded=True | 0.0 | 0.6 | 0.4 |
| adaptive | settled_rate | — | 0.6 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | load[0] graded=True | 0.0 | 0.0 | 1.0 |
| classic | settled_rate | — | 0.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | load[0] graded=True | 0.0 | 0.8 | 0.19999999999999996 |
| edpf | settled_rate | — | 0.8 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | load[0] graded=True | 0.0 | 0.4 | 0.6 |
| rtt-threshold | settled_rate | — | 0.4 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [1.0018298819726128, 1.2968213058419242] | -5.958001778868488 | None | () |
| adaptive | edpf | 5 | [0.821422883534685, 1.2946762237333822] | -0.9276879140763916 | None | () |
| adaptive | enhanced | 5 | [0.8538050418578252, 1.1063453908308958] | 3.8023342614760427 | None | () |
| adaptive | rtt-threshold | 5 | [0.8144540761663582, 1.14921332089889] | -5.1773359521641416 | None | () |
| classic | adaptive | 5 | [0.7711162636634648, 0.9981734603793114] | 5.958001778868488 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.7427251837189727, 0.9983458922992098] | 5.030313864792096 | None | () |
| classic | enhanced | 5 | [0.7311065748446578, 0.9562107214225724] | 9.76033604034453 | None | () |
| classic | rtt-threshold | 5 | [0.7481234936302077, 0.9955475587813729] | 0.7806658267043465 | None | () |
| edpf | adaptive | 5 | [0.7723938863389013, 1.2173997341014844] | 0.9276879140763916 | None | () |
| edpf | classic | 5 | [1.0016568483063328, 1.3463930157757693] | -5.030313864792096 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.732317907534938, 1.0667249499421867] | 4.730022175552434 | None | () |
| edpf | rtt-threshold | 5 | [0.7493630207735568, 1.3403982800158578] | -4.24964803808775 | None | () |
| enhanced | adaptive | 5 | [0.9038768618622548, 1.1712275648127632] | -3.8023342614760427 | None | () |
| enhanced | classic | 5 | [1.0457945906654147, 1.3677896416298478] | -9.76033604034453 | None | () |
| enhanced | edpf | 5 | [0.9374487772637146, 1.3655271702505667] | -4.730022175552434 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.934718028635434, 1.2895611600744106] | -8.979670213640183 | None | () |
| rtt-threshold | adaptive | 5 | [0.8701604670034816, 1.2278163118871084] | 5.1773359521641416 | None | () |
| rtt-threshold | classic | 5 | [1.0044723541124216, 1.3366777123220421] | -0.7806658267043465 | None | () |
| rtt-threshold | edpf | 5 | [0.7460469137487773, 1.3344667034246156] | 4.24964803808775 | None | () |
| rtt-threshold | enhanced | 5 | [0.7754575982594712, 1.069841352541225] | 8.979670213640183 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 13847618.773333335 | 11999288.0 | 4100552.604005832 | [10934205.333333334, 20678044.8] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.38268584588003207 | 0.4673019177643432 | 0.18370989755895248 | [0.07632979365756656, 0.5126890172302235] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.12395375218791371 | 0.10236466830149624 | 0.042343528958148066 | [0.08153490206631567, 0.18550935800218912] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 82.57250550622089 | 111.67329261536185 | 50.88449094174649 | [23.66439080997316, 132.18457942500683] | 0.0 |
| adaptive | switch_count | 5 | 0 | 534.6 | 75.0 | 803.2937196318667 | [7.0, 1869.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 8.909995833402776 | 1.2499791670138831 | 13.388231640403086 | [0.11666666666666668, 31.15] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 12.469944167597205 | 15.65 | 6.021847939700602 | [5.766666666666667, 18.066666666666663] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.5631841654041907 | 0.5781148192008979 | 0.05809692123784469 | [0.46598459773340706, 0.6083725216224202] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.3919247986952 | 18.75126608695652 | 2.998937459298268 | [16.96539032258065, 24.28696245762712] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1586.8 | 1586.0 | 24.772969139770062 | [1560.0, 1616.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 469.3991 | 610.215 | 253.73543495622562 | [48.5735, 645.3005] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 13908.2 | 13449.0 | 2485.4981794400896 | [11041.0, 17827.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 13908.2 | 13449.0 | 2485.4981794400896 | [11041.0, 17827.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 47473.2 | 58359.0 | 22843.30396855936 | [9481.0, 64182.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.2677629178394577 | 0.32625752485739407 | 0.10526066138090005 | [0.11445750542919632, 0.3620885357548241] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.4 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | 1756.0 | None | [1596.0, +inf] | 0.4 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.4 |
| adaptive | episode[3].recovery_ms | 5 | 0 | +inf | 1472.0 | None | [1467.0, +inf] | 0.4 |
| adaptive | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].failover_ms | 5 | 0 | +inf | 2894.0 | None | [0.0, +inf] | 0.4 |
| adaptive | episode[5].recovery_ms | 5 | 0 | +inf | 2357.0 | None | [1749.0, +inf] | 0.4 |
| adaptive | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[7].failover_ms | 5 | 0 | +inf | 2254.0 | None | [0.0, +inf] | 0.4 |
| adaptive | episode[7].recovery_ms | 5 | 0 | +inf | 2149.0 | None | [1449.0, +inf] | 0.4 |
| adaptive | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| adaptive | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [1762.0, +inf] | 0.6 |
| adaptive | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| adaptive | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [1120.0, +inf] | 0.6 |
| adaptive | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 14299269.973333335 | 14486878.933333334 | 1435027.7058168582 | [11999638.933333334, 15919388.8] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.3652222727363787 | 0.35182395218454393 | 0.0674219258932502 | [0.29261319671263225, 0.47531115600156393] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.11937457117596806 | 0.11740973150475403 | 0.03850697179360621 | [0.06902630661452798, 0.17621452016081238] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 64.9975366456741 | 80.1644949666269 | 25.80845050028511 | [28.141783935825476, 87.81032997486234] | 0.0 |
| classic | switch_count | 5 | 0 | 4112.8 | 5084.0 | 2234.999038031113 | [117.0, 5205.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 68.54666666666665 | 84.73333333333333 | 37.249983967185216 | [1.95, 86.75] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 11.603333333333333 | 14.45 | 4.8532835391218505 | [5.6, 16.283333333333335] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.559600709091211 | 0.5512342858757754 | 0.024258261203493114 | [0.5451440778194421, 0.6024943154413285] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.838721003115644 | 20.36151841463415 | 1.3175903558220181 | [17.49162220588235, 20.568863950617285] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1544.0 | 1532.0 | 25.874698065871222 | [1520.0, 1578.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 462.76419999999996 | 447.696 | 82.45720533525252 | [380.806, 582.511] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 12424.2 | 13293.0 | 1883.4437076801632 | [9144.0, 13762.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 12424.2 | 13293.0 | 1883.4437076801632 | [9144.0, 13762.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 44855.4 | 42735.0 | 8256.174101846447 | [35925.0, 58353.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.24540901096233858 | 0.24955288707204903 | 0.044408631241018244 | [0.18952495235406225, 0.3111457791644999] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| classic | episode[1].recovery_ms | 5 | 0 | +inf | 1760.0 | None | [1756.0, +inf] | 0.2 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| classic | episode[3].recovery_ms | 5 | 0 | +inf | 1475.0 | None | [1395.0, +inf] | 0.2 |
| classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 10785655.253333334 | 10843664.533333331 | 355982.6158353441 | [10379379.733333332, 11175822.933333334] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.5173978801465188 | 0.5155664133468368 | 0.016770504760315556 | [0.5011811852394726, 0.5400393302882392] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.04686709789324961 | 0.04458353794046732 | 0.03139100868138595 | [0.008240385440452053, 0.09155579913952275] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 107.05515965037341 | 128.74110169547697 | 36.47701174149227 | [57.81840741509511, 139.50737300320117] | 0.0 |
| edpf | switch_count | 5 | 0 | 7.8 | 8.0 | 0.4472135954999579 | [7.0, 8.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 0.13 | 0.13333333333333333 | 0.007453559924999292 | [0.11666666666666668, 0.13333333333333333] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 14.323333333333332 | 16.833333333333332 | 4.564257271938605 | [8.0, 18.1] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.6187374951226537 | 0.6154112708407588 | 0.007983508675591739 | [0.6115118185443571, 0.6316392276371489] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 16.333482666330646 | 16.447996612903225 | 0.4203041012729304 | [15.630944745762708, 16.67809] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1814.6 | 1829.0 | 44.26398084221526 | [1751.0, 1854.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 610.8735 | 609.719 | 10.596365432307408 | [597.0415, 624.9495] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 9281.6 | 9588.0 | 1248.1868049294546 | [8059.0, 11056.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 9281.6 | 9588.0 | 1248.1868049294546 | [8059.0, 11056.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 66159.2 | 65845.0 | 1788.8484284589344 | [64282.0, 68929.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.329824942982203 | 0.3308578453115693 | 0.023664932959949984 | [0.2987121377486165, 0.35641105246817756] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [1888.0, +inf] | 0.8 |
| edpf | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [1545.0, +inf] | 0.8 |
| edpf | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [1519.0, +inf] | 0.8 |
| edpf | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [1392.0, +inf] | 0.8 |
| edpf | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 11200.0 | 13000.0 | 3834.0579025361626 | [5000.0, 14000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 13792206.4 | 14139630.4 | 2279815.4047430567 | [11244254.933333334, 16918671.466666665] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.38817855357034425 | 0.37556932119863934 | 0.10169291873346847 | [0.2478868897937994, 0.5049087116544654] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.11541999511172334 | 0.11839539259645243 | 0.04397280443602054 | [0.04448605084888521, 0.1621792737666107] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 70.76861662030187 | 62.02220637718159 | 28.573946825454378 | [43.96544106456145, 115.99916400062482] | 0.0 |
| enhanced | switch_count | 5 | 0 | 528.6 | 694.0 | 517.2130122106365 | [7.0, 1207.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 8.809932112242574 | 11.566666666666666 | 8.620107980650596 | [0.1166647222546291, 20.11633139447676] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 11.983207557651818 | 13.116448059199014 | 4.197761608532797 | [7.333211113148114, 17.283045282578623] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.5674854868260333 | 0.5544848276566694 | 0.04259294705343464 | [0.5120107723891305, 0.6217960659138166] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.31679033240546 | 20.182165595238104 | 2.4640112241157794 | [16.356389218750003, 22.429929583333323] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1548.0 | 1553.0 | 66.63707676661694 | [1487.0, 1652.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 384.5603 | 395.9835 | 240.05422207873158 | [49.343999999999994, 616.7349999999999] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 12598.2 | 11144.0 | 2993.8834646659175 | [9774.0, 15854.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 12598.2 | 11144.0 | 2993.8834646659175 | [9774.0, 15854.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 48249.0 | 46260.0 | 12858.125193821998 | [30559.0, 62693.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.2467777725290879 | 0.23249001886772783 | 0.060012064404223096 | [0.1708464517062932, 0.3261284399265509] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | 1965.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | 1754.0 | None | [1731.0, +inf] | 0.4 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 5 | 0 | +inf | 1726.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[3].recovery_ms | 5 | 0 | +inf | 1478.0 | None | [1474.0, +inf] | 0.4 |
| enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| enhanced | episode[5].recovery_ms | 5 | 0 | +inf | 2743.0 | None | [1385.0, +inf] | 0.2 |
| enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[7].failover_ms | 5 | 0 | +inf | 2732.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[7].recovery_ms | 5 | 0 | +inf | 2471.0 | None | [1168.0, +inf] | 0.4 |
| enhanced | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[9].failover_ms | 5 | 0 | +inf | 3493.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[9].recovery_ms | 5 | 0 | +inf | 3369.0 | None | [1403.0, +inf] | 0.4 |
| enhanced | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[11].failover_ms | 5 | 0 | +inf | 3274.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[11].recovery_ms | 5 | 0 | +inf | 3187.0 | None | [1310.0, +inf] | 0.4 |
| enhanced | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 4600.0 | 1000.0 | 8049.844718999243 | [1000.0, 19000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 13743567.040000001 | 13803436.266666668 | 723069.6519053151 | [12834684.8, 14709370.666666666] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.38985788219287026 | 0.38500170379865967 | 0.032881126546983 | [0.34816200783908896, 0.4317128607111018] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.14831182002752868 | 0.15946081548024071 | 0.02630560490837359 | [0.11463925329024593, 0.17618606313148746] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 75.54638969431693 | 72.92738501844626 | 29.911072029973035 | [35.89548539942067, 117.5552692328631] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 180.2 | 218.0 | 97.00360818031461 | [65.0, 287.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 3.0033333333333334 | 3.6333333333333337 | 1.6167268030052435 | [1.0833333333333333, 4.783333333333333] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 12.916666666666666 | 11.7 | 5.139012010537089 | [6.6, 20.283333333333335] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.5653257320505107 | 0.5624068122328617 | 0.015031364097755472 | [0.5509094193597459, 0.5882952268282672] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.530438769409752 | 19.46651368421053 | 0.805097499383676 | [18.393976891891885, 20.409983500000006] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1494.4 | 1492.0 | 36.14277244484711 | [1458.0, 1553.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 460.262 | 533.9135 | 127.60569684530152 | [299.832, 565.7375] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 12450.2 | 13224.0 | 3175.1642792145417 | [8882.0, 15743.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 12450.2 | 13224.0 | 3175.1642792145417 | [8882.0, 15743.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 48226.0 | 47453.0 | 4242.322359274458 | [43170.0, 53753.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.2510691191085096 | 0.25966464834653 | 0.0377830516633934 | [0.1881253447846921, 0.2871385287171947] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | 2954.0 | None | [0.0, +inf] | 0.4 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | 2737.0 | None | [1457.0, +inf] | 0.4 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | +inf | 2680.0 | None | [0.0, +inf] | 0.4 |
| rtt-threshold | episode[3].recovery_ms | 5 | 0 | +inf | 2478.0 | None | [1308.0, +inf] | 0.4 |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [3980.0, +inf] | 0.6 |
| rtt-threshold | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [3764.0, +inf] | 0.6 |
| rtt-threshold | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [3743.0, +inf] | 0.6 |
| rtt-threshold | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [3475.0, +inf] | 0.6 |
| rtt-threshold | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[9].failover_ms | 5 | 0 | +inf | 4979.0 | None | [0.0, +inf] | 0.4 |
| rtt-threshold | episode[9].recovery_ms | 5 | 0 | +inf | 4761.0 | None | [2744.0, +inf] | 0.4 |
| rtt-threshold | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[11].failover_ms | 5 | 0 | +inf | 4741.0 | None | [0.0, +inf] | 0.4 |
| rtt-threshold | episode[11].recovery_ms | 5 | 0 | +inf | 4473.0 | None | [1478.0, +inf] | 0.4 |
| rtt-threshold | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 0.6 | 0.4 |
| adaptive | episode[2] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[3] graded=True | — | 0.6 | 0.4 |
| adaptive | episode[4] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[5] graded=True | — | 0.6 | 0.4 |
| adaptive | episode[6] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[7] graded=True | — | 0.6 | 0.4 |
| adaptive | episode[8] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[9] graded=True | — | 0.4 | 0.6 |
| adaptive | episode[10] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[11] graded=True | — | 0.4 | 0.6 |
| adaptive | episode[12] graded=True | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 0.8 | 0.19999999999999996 |
| classic | episode[2] graded=True | — | 0.0 | 1.0 |
| classic | episode[3] graded=True | — | 0.8 | 0.19999999999999996 |
| classic | episode[4] graded=True | — | 0.0 | 1.0 |
| classic | episode[5] graded=True | — | 0.0 | 1.0 |
| classic | episode[6] graded=True | — | 0.0 | 1.0 |
| classic | episode[7] graded=True | — | 0.0 | 1.0 |
| classic | episode[8] graded=True | — | 0.0 | 1.0 |
| classic | episode[9] graded=True | — | 0.0 | 1.0 |
| classic | episode[10] graded=True | — | 0.0 | 1.0 |
| classic | episode[11] graded=True | — | 0.0 | 1.0 |
| classic | episode[12] graded=True | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 0.0 | 1.0 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | episode[3] graded=True | — | 0.0 | 1.0 |
| edpf | episode[4] graded=True | — | 0.0 | 1.0 |
| edpf | episode[5] graded=True | — | 0.2 | 0.8 |
| edpf | episode[6] graded=True | — | 0.0 | 1.0 |
| edpf | episode[7] graded=True | — | 0.2 | 0.8 |
| edpf | episode[8] graded=True | — | 0.0 | 1.0 |
| edpf | episode[9] graded=True | — | 0.0 | 1.0 |
| edpf | episode[10] graded=True | — | 0.0 | 1.0 |
| edpf | episode[11] graded=True | — | 0.0 | 1.0 |
| edpf | episode[12] graded=True | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 0.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[4] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=True | — | 0.8 | 0.19999999999999996 |
| enhanced | episode[6] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[7] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[8] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[9] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[10] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[11] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[12] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 0.6 | 0.4 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[3] graded=True | — | 0.6 | 0.4 |
| rtt-threshold | episode[4] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[5] graded=True | — | 0.4 | 0.6 |
| rtt-threshold | episode[6] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[7] graded=True | — | 0.4 | 0.6 |
| rtt-threshold | episode[8] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[9] graded=True | — | 0.6 | 0.4 |
| rtt-threshold | episode[10] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[11] graded=True | — | 0.6 | 0.4 |
| rtt-threshold | episode[12] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [0.7594656137205978, 1.2989220289663381] | 11.547796557979929 | 0.9955978104953926 | () |
| adaptive | edpf | 5 | [1.008349649670707, 1.8502480688312504] | -4.8264495582493625 | 0.0 | () |
| adaptive | enhanced | 5 | [0.6462803746071915, 1.402961975285126] | 9.173259656570387 | 1.7708511572127987 | () |
| adaptive | rtt-threshold | 5 | [0.7762207274539114, 1.6111065540152572] | 8.230021396568354 | 1.0007507257015114 | () |
| classic | adaptive | 5 | [0.7698691512652105, 1.3167153086774157] | -11.547796557979929 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [1.106603666725999, 1.4244489103812097] | -16.37424611622929 | None | () |
| classic | enhanced | 5 | [0.7092542081081923, 1.3193408445429293] | -2.3745369014095408 | None | () |
| classic | rtt-threshold | 5 | [0.8518560039860489, 1.240341235337544] | -3.3177751614115736 | None | () |
| edpf | adaptive | 5 | [0.540468068496173, 0.9917194896894806] | 4.8264495582493625 | None | () |
| edpf | classic | 5 | [0.7020258801225668, 0.9036659014140113] | 16.37424611622929 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.6409288433017704, 0.9844262039262195] | 13.99970921481975 | None | () |
| edpf | rtt-threshold | 5 | [0.7525229631396875, 0.8707516473901512] | 13.056470954817717 | None | () |
| enhanced | adaptive | 5 | [0.7127776929212701, 1.547316055524352] | -9.173259656570387 | 1.5694297277625244 | () |
| enhanced | classic | 5 | [0.7579542497575285, 1.409931712167517] | 2.3745369014095408 | 0.49176136363636364 | () |
| enhanced | edpf | 5 | [1.015820176272906, 1.5602356025178403] | -13.99970921481975 | 0.0 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [0.7644280090659669, 1.201058794220229] | -0.9432382600020328 | 1.068243619188661 | () |
| rtt-threshold | adaptive | 5 | [0.6206914108242961, 1.2882933483110004] | -8.230021396568354 | 2.2696375280586585 | () |
| rtt-threshold | classic | 5 | [0.8062297467098737, 1.1739073215669644] | 3.3177751614115736 | 1.6166742338251985 | () |
| rtt-threshold | edpf | 5 | [1.148433084217798, 1.3288631031640352] | -13.056470954817717 | 0.0 | () |
| rtt-threshold | enhanced | 5 | [0.8325987077503864, 1.3081676601853873] | 0.9432382600020328 | 3.602799853602426 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--D@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 13250989.994666666 | 13490158.08 | 442344.59856543015 | [12614509.226666668, 13597543.68] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.41110541773114173 | 0.4011204661139034 | 0.018631313291908428 | [0.396098012376858, 0.43982902235861465] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.08109437602768614 | 0.07940762239646737 | 0.028072300571822546 | [0.05015611481424673, 0.12484939048011612] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 72.35030080394378 | 72.27893585566991 | 34.12103893017499 | [36.55562198323945, 107.81235920974267] | 0.0 |
| adaptive | switch_count | 5 | 0 | 721.0 | 762.0 | 124.45481107614924 | [582.0, 858.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 9.613282134015991 | 10.16 | 1.659384395228681 | [7.759896534712871, 11.439847468700416] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 11.853284018435309 | 12.279836268849747 | 5.275462994410563 | [6.213250489993467, 17.026666666666667] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.5732271533958231 | 0.5744133055485228 | 0.007052145075066428 | [0.5629159173514838, 0.5820991808599179] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.369750298324803 | 19.289322514583336 | 0.2125536212790639 | [19.195458043478272, 19.72524670103092] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1449.6 | 1452.0 | 13.049904214207858 | [1431.0, 1466.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 401.9306 | 555.7555 | 222.11959137517562 | [99.061, 568.9855] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 15012.2 | 14283.0 | 2671.4224113756327 | [12463.0, 18754.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 15012.2 | 14283.0 | 2671.4224113756327 | [12463.0, 18754.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 63781.8 | 62649.0 | 2699.4737820545693 | [61638.0, 68221.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.25204196141911545 | 0.24379361827714316 | 0.024825135829060756 | [0.2333202170590873, 0.29564445420682206] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.4 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | 1765.0 | None | [1681.0, +inf] | 0.4 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.4 |
| adaptive | episode[3].recovery_ms | 5 | 0 | +inf | 1478.0 | None | [1448.0, +inf] | 0.4 |
| adaptive | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].failover_ms | 5 | 0 | 15574.0 | 11986.0 | 7215.525448641977 | [10985.0, 27949.0] | 0.0 |
| adaptive | episode[5].recovery_ms | 5 | 0 | 7575.8 | 3990.0 | 7234.0922512779725 | [2965.0, 19986.0] | 0.0 |
| adaptive | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[7].failover_ms | 5 | 0 | +inf | 1980.0 | None | [0.0, +inf] | 0.2 |
| adaptive | episode[7].recovery_ms | 5 | 0 | +inf | 1765.0 | None | [1590.0, +inf] | 0.2 |
| adaptive | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[9].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| adaptive | episode[9].recovery_ms | 5 | 0 | +inf | 1476.0 | None | [1415.0, +inf] | 0.2 |
| adaptive | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| adaptive | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [1424.0, +inf] | 0.6 |
| adaptive | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| adaptive | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [1281.0, +inf] | 0.6 |
| adaptive | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[15].failover_ms | 5 | 0 | 1335.4 | 1729.0 | 1304.6402952538297 | [0.0, 2971.0] | 0.0 |
| adaptive | episode[15].recovery_ms | 5 | 0 | 2333.4 | 1763.0 | 894.2213372538143 | [1708.0, 3762.0] | 0.0 |
| adaptive | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[17].failover_ms | 5 | 0 | +inf | 1744.0 | None | [0.0, +inf] | 0.2 |
| adaptive | episode[17].recovery_ms | 5 | 0 | +inf | 2400.0 | None | [1473.0, +inf] | 0.2 |
| adaptive | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 13588868.608000001 | 13820737.28 | 641510.9204454503 | [12794748.586666666, 14298848.853333334] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.39675789988364063 | 0.3901857689127739 | 0.02808736686325719 | [0.364893884578924, 0.43288540813232607] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.10848419067119883 | 0.10522864132039489 | 0.03764043818688589 | [0.05661334746325766, 0.16246720162993947] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 81.08071990574717 | 87.6527436711213 | 26.95971105088474 | [35.8109211763653, 106.40195278416208] | 0.0 |
| classic | switch_count | 5 | 0 | 5206.6 | 5290.0 | 245.13220922596037 | [4793.0, 5430.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 69.4209511873175 | 70.53333333333333 | 3.2681021368412995 | [63.906666666666666, 72.3990346795376] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 13.735914276698532 | 15.666457780562926 | 4.522887377803464 | [6.1866666666666665, 17.346666666666668] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.5688485525801068 | 0.5713362135711852 | 0.010490671686717397 | [0.5564370622943933, 0.5829163378058406] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.583313367568845 | 19.488236021505383 | 0.27038600865597523 | [19.276290219780225, 19.98614628431372] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1433.8 | 1429.0 | 16.05303709582707 | [1418.0, 1459.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 522.349 | 536.864 | 50.2561514729789 | [433.742, 556.152] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 14898.6 | 14916.0 | 1549.4262809181985 | [12552.0, 16744.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 14898.6 | 14916.0 | 1549.4262809181985 | [12552.0, 16744.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 61505.2 | 60407.0 | 4269.7316894624655 | [56583.0, 67102.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.24991655083006226 | 0.2524270967858465 | 0.009140502045985033 | [0.23934491458749696, 0.26221786230750366] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[1].recovery_ms | 5 | 0 | 1638.8 | 1695.0 | 142.69968465276997 | [1407.0, 1759.0] | 0.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[3].recovery_ms | 5 | 0 | 1409.4 | 1415.0 | 78.42703615463229 | [1285.0, 1479.0] | 0.0 |
| classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].failover_ms | 5 | 0 | +inf | 25985.0 | None | [11941.0, +inf] | 0.2 |
| classic | episode[5].recovery_ms | 5 | 0 | +inf | 17943.0 | None | [3988.0, +inf] | 0.2 |
| classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[7].failover_ms | 5 | 0 | 1186.4 | 1976.0 | 1083.0306089857295 | [0.0, 1979.0] | 0.0 |
| classic | episode[7].recovery_ms | 5 | 0 | 1732.4 | 1760.0 | 39.62701098998005 | [1678.0, 1761.0] | 0.0 |
| classic | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[9].failover_ms | 5 | 0 | 1224.8 | 1734.0 | 1179.5476675404007 | [0.0, 2655.0] | 0.0 |
| classic | episode[9].recovery_ms | 5 | 0 | 1622.4 | 1441.0 | 435.499483352162 | [1396.0, 2399.0] | 0.0 |
| classic | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[11].failover_ms | 5 | 0 | +inf | 4896.0 | None | [1898.0, +inf] | 0.4 |
| classic | episode[11].recovery_ms | 5 | 0 | +inf | 4742.0 | None | [1401.0, +inf] | 0.4 |
| classic | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| classic | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [1206.0, +inf] | 0.6 |
| classic | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| classic | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [1758.0, +inf] | 0.6 |
| classic | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.8 |
| classic | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [1478.0, +inf] | 0.8 |
| classic | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 10499363.84 | 10500206.08 | 322896.30588714883 | [10120917.333333334, 10932415.573333334] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.5308577691306162 | 0.5337843365532866 | 0.014060332872819551 | [0.5110348455622635, 0.5460034538830975] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.03778273904851847 | 0.03394625861225864 | 0.03012990303555398 | [0.011210397593379717, 0.08551162056445705] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 106.43151944504723 | 105.19318683808461 | 33.669613889486904 | [56.27826052470518, 150.50019181396996] | 0.0 |
| edpf | switch_count | 5 | 0 | 9.8 | 10.0 | 1.4832396974191326 | [8.0, 12.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 0.13066638222601476 | 0.13333333333333333 | 0.0197769607866011 | [0.10666524446340717, 0.16] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 13.935949227343636 | 14.04 | 4.252393958232976 | [7.386666666666667, 19.039746136718176] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.6266249832083725 | 0.6265145968038853 | 0.007058670406045791 | [0.6169792694965449, 0.6368296529968455] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 15.849213170146054 | 15.746309864864868 | 0.4552150205569025 | [15.401940972222224, 16.507525769230774] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1786.4 | 1771.0 | 23.19051530259731 | [1767.0, 1816.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 602.8924 | 598.912 | 18.485348831574715 | [576.2275, 621.8945] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 10309.8 | 9853.0 | 1655.2646918242413 | [8785.0, 12951.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 10309.8 | 9853.0 | 1655.2646918242413 | [8785.0, 12951.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 84454.0 | 84821.0 | 2021.3053950355943 | [81600.0, 86630.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.332643202678942 | 0.326368193684016 | 0.015535130901503786 | [0.31918742325907734, 0.35634615597620817] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [3976.0, +inf] | 0.8 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [3763.0, +inf] | 0.8 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [3739.0, +inf] | 0.8 |
| edpf | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [3474.0, +inf] | 0.8 |
| edpf | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].failover_ms | 5 | 0 | 8172.2 | 6989.0 | 1779.8619890317339 | [6947.0, 10948.0] | 0.0 |
| edpf | episode[5].recovery_ms | 5 | 0 | 2635.2 | 2566.0 | 822.7713534147867 | [1956.0, 3984.0] | 0.0 |
| edpf | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[7].failover_ms | 5 | 0 | 1086.2 | 0.0 | 1600.6399345261882 | [0.0, 3552.0] | 0.0 |
| edpf | episode[7].recovery_ms | 5 | 0 | 2065.0 | 1728.0 | 771.5837608451852 | [1671.0, 3444.0] | 0.0 |
| edpf | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[9].failover_ms | 5 | 0 | 996.0 | 0.0 | 1487.840885309985 | [0.0, 3331.0] | 0.0 |
| edpf | episode[9].recovery_ms | 5 | 0 | 1806.0 | 1476.0 | 802.5929852671278 | [1381.0, 3240.0] | 0.0 |
| edpf | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[11].failover_ms | 5 | 0 | 1699.8 | 2567.0 | 1560.247480369701 | [0.0, 2978.0] | 0.0 |
| edpf | episode[11].recovery_ms | 5 | 0 | 2274.8 | 2457.0 | 482.4056384413432 | [1759.0, 2764.0] | 0.0 |
| edpf | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[13].failover_ms | 5 | 0 | 1541.0 | 2353.0 | 1413.6949812459545 | [0.0, 2743.0] | 0.0 |
| edpf | episode[13].recovery_ms | 5 | 0 | 2027.4 | 2243.0 | 513.3705289554514 | [1473.0, 2479.0] | 0.0 |
| edpf | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[15].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.4 |
| edpf | episode[15].recovery_ms | 5 | 0 | +inf | 2763.0 | None | [1376.0, +inf] | 0.4 |
| edpf | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[17].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.4 |
| edpf | episode[17].recovery_ms | 5 | 0 | +inf | 2475.0 | None | [1161.0, +inf] | 0.4 |
| edpf | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 8600.0 | 10000.0 | 3435.1128074635335 | [3000.0, 12000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 12271717.546666667 | 13145822.293333331 | 1392033.3312065888 | [10635947.093333334, 13458433.706666669] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.45381357867859756 | 0.4123639928116321 | 0.06161159799147916 | [0.4025003379705032, 0.5245917014731261] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.07000086341422794 | 0.07867669887672087 | 0.028626891669326 | [0.03150147184747831, 0.10238202443019895] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 90.25143094068775 | 96.77203368433973 | 20.456501341146186 | [56.01479182902128, 109.11424466005117] | 0.0 |
| enhanced | switch_count | 5 | 0 | 389.0 | 501.0 | 336.1398518474119 | [13.0, 695.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 5.186624427229859 | 6.679910934520873 | 4.4818302039882765 | [0.17333333333333334, 9.266666666666667] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 13.927916694444074 | 14.626471647044706 | 3.655114989400015 | [7.613333333333333, 16.613111825175665] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.5873256048246478 | 0.5738064999285849 | 0.027698077396232822 | [0.5616451476954906, 0.6238099785349813] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 18.3164900984258 | 19.001111382978724 | 1.824790585390356 | [16.32841337662337, 20.136295625000013] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1517.0 | 1440.0 | 140.95566678924263 | [1422.0, 1757.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 577.7224 | 574.4875 | 26.05902061043354 | [551.6315, 608.194] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 15201.2 | 14660.0 | 2648.938504382463 | [11541.0, 17945.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 15201.2 | 14660.0 | 2648.938504382463 | [11541.0, 17945.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 70634.2 | 63700.0 | 10407.413809395684 | [62524.0, 83899.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.30807783898159014 | 0.2932838186676328 | 0.04872274476630062 | [0.2670348742610236, 0.3925234610773492] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | 1924.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | 1758.0 | None | [1693.0, +inf] | 0.4 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 5 | 0 | +inf | 1719.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[3].recovery_ms | 5 | 0 | +inf | 1407.0 | None | [1396.0, +inf] | 0.4 |
| enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 5 | 0 | 16354.0 | 14898.0 | 10502.850160789689 | [6989.0, 33964.0] | 0.0 |
| enhanced | episode[5].recovery_ms | 5 | 0 | 8974.4 | 6990.0 | 9894.663526366117 | [1948.0, 25960.0] | 0.0 |
| enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[7].failover_ms | 5 | 0 | +inf | 4969.0 | None | [1956.0, +inf] | 0.2 |
| enhanced | episode[7].recovery_ms | 5 | 0 | +inf | 4604.0 | None | [1483.0, +inf] | 0.2 |
| enhanced | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[9].failover_ms | 5 | 0 | +inf | 3582.0 | None | [1387.0, +inf] | 0.2 |
| enhanced | episode[9].recovery_ms | 5 | 0 | +inf | 3476.0 | None | [1326.0, +inf] | 0.2 |
| enhanced | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[11].failover_ms | 5 | 0 | +inf | 4975.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[11].recovery_ms | 5 | 0 | +inf | 4704.0 | None | [1758.0, +inf] | 0.4 |
| enhanced | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[13].failover_ms | 5 | 0 | +inf | 4679.0 | None | [0.0, +inf] | 0.4 |
| enhanced | episode[13].recovery_ms | 5 | 0 | +inf | 4442.0 | None | [1478.0, +inf] | 0.4 |
| enhanced | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| enhanced | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [1761.0, +inf] | 0.6 |
| enhanced | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| enhanced | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [1481.0, +inf] | 0.6 |
| enhanced | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1800.0 | 1000.0 | 1788.8543819998317 | [1000.0, 5000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 13498805.077333331 | 13641480.533333331 | 802495.3210296789 | [12283368.533333331, 14332117.333333334] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.39847278163793487 | 0.3924096486858873 | 0.0364374439997188 | [0.3578650410194883, 0.4522855635247904] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.1507041706019188 | 0.1521085871842152 | 0.03012260164760011 | [0.11128425280756862, 0.18129576197240418] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 86.79208290084237 | 86.18405580082701 | 7.253325007145306 | [78.32048295465016, 97.11555844417435] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 200.2 | 217.0 | 101.0801662048495 | [59.0, 321.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 2.6693127824962333 | 2.8933333333333335 | 1.3477120813491517 | [0.7866666666666666, 4.279942934094212] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 14.671913672262146 | 15.44 | 1.7824552776517035 | [12.626666666666669, 16.55977920294396] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.5686780121116168 | 0.569346279426777 | 0.013430960683391844 | [0.5544994890613544, 0.5896622990950499] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 19.08523031326974 | 19.24484542553191 | 0.9140216557542329 | [17.538103448275873, 19.951663499999995] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1437.0 | 1440.0 | 6.082762530298219 | [1429.0, 1442.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 563.0485000000001 | 558.423 | 19.30257350070192 | [539.626, 592.439] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 13359.0 | 13361.0 | 2168.5686523603536 | [10312.0, 15794.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 13359.0 | 13361.0 | 2168.5686523603536 | [10312.0, 15794.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 61567.4 | 60663.0 | 5475.6484821434615 | [55181.0, 69429.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.26686305945890304 | 0.2685177645685748 | 0.024291970642624925 | [0.2392786626299855, 0.2954523227383863] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | 395.4 | 0.0 | 884.1412783034169 | [0.0, 1977.0] | 0.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | 1680.0 | 1756.0 | 112.67874688689079 | [1517.0, 1762.0] | 0.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| rtt-threshold | episode[3].recovery_ms | 5 | 0 | +inf | 1477.0 | None | [1332.0, +inf] | 0.2 |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].failover_ms | 5 | 0 | +inf | 12985.0 | None | [2987.0, +inf] | 0.2 |
| rtt-threshold | episode[5].recovery_ms | 5 | 0 | +inf | 4980.0 | None | [1987.0, +inf] | 0.2 |
| rtt-threshold | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[7].failover_ms | 5 | 0 | 1964.8 | 1975.0 | 1379.9741664248647 | [0.0, 3903.0] | 0.0 |
| rtt-threshold | episode[7].recovery_ms | 5 | 0 | 2059.2 | 1717.0 | 803.8085592975482 | [1669.0, 3496.0] | 0.0 |
| rtt-threshold | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[9].failover_ms | 5 | 0 | +inf | 1582.0 | None | [0.0, +inf] | 0.2 |
| rtt-threshold | episode[9].recovery_ms | 5 | 0 | +inf | 1479.0 | None | [1470.0, +inf] | 0.2 |
| rtt-threshold | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| rtt-threshold | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [1760.0, +inf] | 0.6 |
| rtt-threshold | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| rtt-threshold | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [1474.0, +inf] | 0.6 |
| rtt-threshold | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[15].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| rtt-threshold | episode[15].recovery_ms | 5 | 0 | +inf | 1685.0 | None | [1576.0, +inf] | 0.2 |
| rtt-threshold | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[17].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| rtt-threshold | episode[17].recovery_ms | 5 | 0 | +inf | 1398.0 | None | [1397.0, +inf] | 0.2 |
| rtt-threshold | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 0.6 | 0.4 |
| adaptive | episode[2] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[3] graded=True | — | 0.6 | 0.4 |
| adaptive | episode[4] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[5] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[6] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[7] graded=True | — | 0.8 | 0.19999999999999996 |
| adaptive | episode[8] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[9] graded=True | — | 0.8 | 0.19999999999999996 |
| adaptive | episode[10] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[11] graded=True | — | 0.4 | 0.6 |
| adaptive | episode[12] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[13] graded=True | — | 0.4 | 0.6 |
| adaptive | episode[14] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[15] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[16] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[17] graded=True | — | 0.8 | 0.19999999999999996 |
| adaptive | episode[18] graded=True | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 1.0 | 0.0 |
| classic | episode[2] graded=True | — | 0.0 | 1.0 |
| classic | episode[3] graded=True | — | 1.0 | 0.0 |
| classic | episode[4] graded=True | — | 0.0 | 1.0 |
| classic | episode[5] graded=True | — | 0.8 | 0.19999999999999996 |
| classic | episode[6] graded=True | — | 0.0 | 1.0 |
| classic | episode[7] graded=True | — | 1.0 | 0.0 |
| classic | episode[8] graded=True | — | 0.0 | 1.0 |
| classic | episode[9] graded=True | — | 1.0 | 0.0 |
| classic | episode[10] graded=True | — | 0.0 | 1.0 |
| classic | episode[11] graded=True | — | 0.6 | 0.4 |
| classic | episode[12] graded=True | — | 0.0 | 1.0 |
| classic | episode[13] graded=True | — | 0.4 | 0.6 |
| classic | episode[14] graded=True | — | 0.0 | 1.0 |
| classic | episode[15] graded=True | — | 0.4 | 0.6 |
| classic | episode[16] graded=True | — | 0.0 | 1.0 |
| classic | episode[17] graded=True | — | 0.2 | 0.8 |
| classic | episode[18] graded=True | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 0.2 | 0.8 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | episode[3] graded=True | — | 0.2 | 0.8 |
| edpf | episode[4] graded=True | — | 0.0 | 1.0 |
| edpf | episode[5] graded=True | — | 1.0 | 0.0 |
| edpf | episode[6] graded=True | — | 0.0 | 1.0 |
| edpf | episode[7] graded=True | — | 1.0 | 0.0 |
| edpf | episode[8] graded=True | — | 0.0 | 1.0 |
| edpf | episode[9] graded=True | — | 1.0 | 0.0 |
| edpf | episode[10] graded=True | — | 0.0 | 1.0 |
| edpf | episode[11] graded=True | — | 1.0 | 0.0 |
| edpf | episode[12] graded=True | — | 0.0 | 1.0 |
| edpf | episode[13] graded=True | — | 1.0 | 0.0 |
| edpf | episode[14] graded=True | — | 0.0 | 1.0 |
| edpf | episode[15] graded=True | — | 0.6 | 0.4 |
| edpf | episode[16] graded=True | — | 0.0 | 1.0 |
| edpf | episode[17] graded=True | — | 0.6 | 0.4 |
| edpf | episode[18] graded=True | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 0.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[4] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[6] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[7] graded=True | — | 0.8 | 0.19999999999999996 |
| enhanced | episode[8] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[9] graded=True | — | 0.8 | 0.19999999999999996 |
| enhanced | episode[10] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[11] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[12] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[13] graded=True | — | 0.6 | 0.4 |
| enhanced | episode[14] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[15] graded=True | — | 0.4 | 0.6 |
| enhanced | episode[16] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[17] graded=True | — | 0.4 | 0.6 |
| enhanced | episode[18] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 0.8 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[3] graded=True | — | 0.8 | 0.19999999999999996 |
| rtt-threshold | episode[4] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[5] graded=True | — | 0.8 | 0.19999999999999996 |
| rtt-threshold | episode[6] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[7] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[8] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[9] graded=True | — | 0.8 | 0.19999999999999996 |
| rtt-threshold | episode[10] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[11] graded=True | — | 0.4 | 0.6 |
| rtt-threshold | episode[12] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[13] graded=True | — | 0.4 | 0.6 |
| rtt-threshold | episode[14] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[15] graded=True | — | 0.8 | 0.19999999999999996 |
| rtt-threshold | episode[16] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[17] graded=True | — | 0.8 | 0.19999999999999996 |
| rtt-threshold | episode[18] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [0.8822045296133043, 1.0543511651380173] | 1.0934697201129495 | 1.023321554770318 | () |
| adaptive | edpf | 5 | [1.1538629447490403, 1.3140895914237272] | -13.266387043938321 | 1.0538599640933572 | () |
| adaptive | enhanced | 5 | [0.963035587634027, 1.250542215336948] | -1.1243526697728712 | 1.0372492836676217 | () |
| adaptive | rtt-threshold | 5 | [0.880156709108717, 1.1069881721044514] | 0.8710817428016104 | 1.5676136363636364 | () |
| classic | adaptive | 5 | [0.948450604565982, 1.1335239918098459] | -1.0934697201129495 | 0.9994321408290744 | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| classic | edpf | 5 | [1.2463490674396982, 1.3162348600304803] | -14.35985676405127 | 1.0020325203252032 | () |
| classic | enhanced | 5 | [0.9690850682131085, 1.3443888661589831] | -2.2178223898858205 | 1.001366742596811 | () |
| classic | rtt-threshold | 5 | [0.9289435007348604, 1.1251585623678648] | -0.22238797731133908 | 1.0020338983050847 | () |
| edpf | adaptive | 5 | [0.7609831220994359, 0.8666540550164693] | 13.266387043938321 | 0.9988668555240793 | () |
| edpf | classic | 5 | [0.7597428318961578, 0.8023434414359063] | 14.35985676405127 | 1.0429799426934097 | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| edpf | enhanced | 5 | [0.7520130168133838, 1.027874196571157] | 12.14203437416545 | 0.5597363945578231 | () |
| edpf | rtt-threshold | 5 | [0.7208630360231557, 0.8548311525055713] | 14.137468786739932 | 1.0011648223645895 | () |
| enhanced | adaptive | 5 | [0.7996531326457927, 1.0383832257505528] | 1.1243526697728712 | 1.666908037653874 | () |
| enhanced | classic | 5 | [0.7438324023443252, 1.031901153779921] | 2.2178223898858205 | 1.0980637101811368 | () |
| enhanced | edpf | 5 | [0.9728817041383649, 1.3297642163661583] | -12.14203437416545 | 2.295192958700068 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [0.7421057786483839, 0.9961175995064563] | 1.9954344125744816 | 2.3614130434782608 | () |
| rtt-threshold | adaptive | 5 | [0.9033520187473545, 1.1361613104246415] | -0.8710817428016104 | 0.9875354107648725 | () |
| rtt-threshold | classic | 5 | [0.8887636226982336, 1.0764917341140003] | 0.22238797731133908 | 1.0005675368898979 | () |
| rtt-threshold | edpf | 5 | [1.1698216625223923, 1.3872260748959777] | -14.137468786739932 | 1.0006788866259335 | () |
| rtt-threshold | enhanced | 5 | [1.003897532274771, 1.3475167944673943] | -1.9954344125744816 | 0.8703384968445209 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--E@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 9417534.634666666 | 9412453.12 | 37607.13991302898 | [9372446.72, 9464110.506666666] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.34693465030623577 | 0.34769006469641284 | 0.0019228209787679113 | [0.34490668857122825, 0.34921500878947687] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.003856409730459487 | 0.0030803169785233175 | 0.002422185143870502 | [0.0017278494190943183, 0.00801597237886209] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 80.54401396558725 | 87.33059907927884 | 18.03599146110282 | [48.95641913167904, 94.44803777777247] | 0.0 |
| adaptive | switch_count | 5 | 0 | 911.4 | 910.0 | 7.987490219086343 | [902.0, 924.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 12.151934791980551 | 12.133171557712563 | 0.10644219973970573 | [12.026666666666667, 12.319835735523526] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 9.482612729608048 | 10.253196624045012 | 2.129839647857664 | [5.76, 11.173333333333334] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.501219888302065 | 0.502082658897162 | 0.002573372014481713 | [0.49675261061079945, 0.5032735287127001] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.9626516996834 | 10.94608597014925 | 0.03644946741388761 | [10.93291537313433, 11.024304545454548] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1457.4 | 1458.0 | 7.092249290598858 | [1447.0, 1467.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 544.823 | 545.817 | 6.990699964953423 | [535.575, 554.111] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 1224.6 | 1048.0 | 433.5825180977665 | [954.0, 1990.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 1224.6 | 1048.0 | 433.5825180977665 | [954.0, 1990.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 34532.0 | 34431.0 | 226.45198166498787 | [34341.0, 34910.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.18450418538091393 | 0.1866123416121249 | 0.00852908248007236 | [0.17007116975357653, 0.19282321128407512] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [50921.0, +inf] | 0.8 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [20895.0, +inf] | 0.8 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 9439601.322666666 | 9423823.36 | 44729.407050894 | [9398135.04, 9512539.306666669] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.3453286837247741 | 0.34644517214265097 | 0.003691769516245356 | [0.3403390571587567, 0.3492856290046564] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.004724792345743323 | 0.0028542546271981273 | 0.005058363416494762 | [0.0008487107659587156, 0.013397730692845444] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 87.42264148085795 | 86.24949438780652 | 8.2497263664705 | [76.02409784888286, 97.33117906990323] | 0.0 |
| classic | switch_count | 5 | 0 | 6298.8 | 3248.0 | 5127.991926670712 | [2982.0, 14878.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 83.98374066123563 | 43.306666666666665 | 68.37316277836001 | [39.76, 198.3733333333333] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 10.317306773687463 | 10.16 | 1.0006072362596443 | [8.946666666666667, 11.573333333333334] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.5027166817512974 | 0.5026662002898199 | 0.0029351774198052584 | [0.49868009526268975, 0.5059166390955743] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.997908350746268 | 10.9965425 | 0.02019317018797978 | [10.968864776119402, 11.021213432835816] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1445.2 | 1446.0 | 15.738487856207787 | [1426.0, 1462.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 552.0857 | 556.8135 | 6.919080444683361 | [542.801, 557.196] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 1111.0 | 1036.0 | 219.25441842754276 | [877.0, 1455.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 1111.0 | 1036.0 | 219.25441842754276 | [877.0, 1455.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 34668.6 | 34807.0 | 457.1239438051785 | [34123.0, 35106.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.1893243909255955 | 0.18985216018372328 | 0.003022308878524035 | [0.18514393172284105, 0.19336394228027148] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 9102171.904 | 9325421.653333334 | 478909.0250696 | [8251846.4, 9364445.44] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.3688419233684061 | 0.35355056315123773 | 0.034310109548398325 | [0.35109997494362316, 0.4299917066165018] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.011686231213685838 | 0.005159469947422113 | 0.014200570281849494 | [0.002855004248143417, 0.0364194613122526] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 91.20259760402105 | 90.55528225705591 | 6.604765671728571 | [85.6538375998677, 102.14372805253842] | 0.0 |
| edpf | switch_count | 5 | 0 | 1961.8 | 2132.0 | 765.4940888080065 | [760.0, 2830.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 26.157257529899603 | 28.426287649498008 | 10.20656678448434 | [10.133333333333333, 37.733333333333334] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 10.389306631467136 | 10.52 | 1.0843813356457916 | [8.906666666666666, 11.906666666666666] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.5289817155618928 | 0.5205190264891757 | 0.02990399750464863 | [0.5076731705583396, 0.5805307732737228] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.945237817136888 | 11.014979696969698 | 0.5636147269401741 | [9.999526206896553, 11.469887575757577] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1485.6 | 1480.0 | 38.69496091224282 | [1447.0, 1537.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 560.4655999999999 | 558.785 | 7.902957542907588 | [551.2175, 572.588] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 1983.8 | 2069.0 | 380.34550082786575 | [1545.0, 2370.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 1983.8 | 2069.0 | 380.34550082786575 | [1545.0, 2370.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 36722.8 | 35220.0 | 3264.6388314789124 | [34882.0, 42515.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.2166887881960875 | 0.2102709960585017 | 0.020373809906318138 | [0.20067223968821987, 0.25194290245836637] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [25935.0, +inf] | 0.8 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [9639.0, +inf] | 0.8 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 10369406.208 | 9366129.92 | 2254613.547087336 | [9292433.92, 14401461.76] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.28041581659796555 | 0.3479323833473037 | 0.15682042178691724 | [0.0, 0.3570533351900849] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.004273403341360166 | 0.003184000646871521 | 0.0029801668627285455 | [0.0015274462962964197, 0.009227336748269366] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 74.21212932053785 | 85.02439435469064 | 29.322281712399796 | [24.812296091069395, 100.210559258946] | 0.0 |
| enhanced | switch_count | 5 | 0 | 1486.4 | 913.0 | 1287.7596048952616 | [903.0, 3790.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 19.818537175059888 | 12.173171024386342 | 17.17020044996158 | [12.039839468807084, 50.53333333333333] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 8.994558650329108 | 9.933200890654792 | 2.771309823734432 | [4.466666666666667, 11.639844802069303] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.4047633655694612 | 0.4988440385416218 | 0.21305604729295458 | [0.023646216226457992, 0.5020907832506633] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 11.777348040674823 | 11.006735970149252 | 1.7422168194383736 | [10.931198030303026, 14.892830097087376] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1548.4 | 1433.0 | 244.89855042445637 | [1432.0, 1986.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 453.9826999999999 | 549.059 | 219.43006966161224 | [61.559, 559.0645] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 1331.2 | 1318.0 | 888.6713115657554 | [79.0, 2572.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 1331.2 | 1318.0 | 888.6713115657554 | [79.0, 2572.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 28037.2 | 34867.0 | 15681.918814354318 | [0.0, 35896.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.1539349861116145 | 0.18836676217765044 | 0.0849047720894988 | [0.002736270133902581, 0.20502396439575488] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.8 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [1959.0, +inf] | 0.8 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 9437523.797333334 | 9452319.146666666 | 76206.08278097304 | [9313770.666666666, 9521803.946666667] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.34597372564930606 | 0.34654698334263173 | 0.005321839984373362 | [0.3396902370231085, 0.3537786202424303] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.0064538031503182935 | 0.006056917573001086 | 0.005203441826270048 | [0.00031474838860626186, 0.01254355828734699] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 91.20534678930608 | 90.1648212933242 | 6.348173701371544 | [83.40873271800444, 98.64158697741662] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 852.4 | 852.0 | 24.825390228554316 | [824.0, 881.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 11.365272711919397 | 11.36 | 0.33100199994679724 | [10.986520179730936, 11.746510046532713] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 10.757274525228553 | 10.653333333333334 | 0.7157835279707614 | [9.84, 11.666511113185155] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.4996835128249124 | 0.5003755163349606 | 0.0030611149371587987 | [0.4965265292620133, 0.5036626113776559] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.977165419626468 | 10.968344545454546 | 0.05227431437489735 | [10.926359402985074, 11.05103328358209] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1453.0 | 1455.0 | 4.69041575982343 | [1445.0, 1457.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 556.6407 | 557.8365 | 4.829460238991496 | [549.821, 561.178] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 995.8 | 713.0 | 437.37821162010346 | [653.0, 1578.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 995.8 | 713.0 | 437.37821162010346 | [653.0, 1578.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 34692.2 | 34785.0 | 533.6100636232417 | [34070.0, 35461.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.19629414560215933 | 0.19539820994066337 | 0.005162830862577984 | [0.19058099224672392, 0.20202828167404657] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 0.2 | 0.8 |
| adaptive | episode[2] graded=True | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 0.0 | 1.0 |
| classic | episode[2] graded=True | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 0.2 | 0.8 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 0.2 | 0.8 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 0.2 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9894784995425432, 1.007020059446461] | 0.12448925537618671 | None | () |
| adaptive | edpf | 5 | [1.0008544318028512, 1.1406481245215614] | -0.5860498454824892 | None | () |
| adaptive | enhanced | 5 | [0.6507982922977953, 1.0129157980603642] | -0.024231865089086035 | None | () |
| adaptive | rtt-threshold | 5 | [0.9843141879938672, 1.0084702336096458] | 0.11430813537811058 | None | () |
| classic | adaptive | 5 | [0.993028878242684, 1.0106333795654188] | -0.12448925537618671 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [1.006340783379053, 1.1527770689801822] | -0.7105391008586759 | None | () |
| classic | enhanced | 5 | [0.6543657523831803, 1.023686516208949] | -0.14872112046527275 | None | () |
| classic | rtt-threshold | 5 | [0.9897098714471045, 1.0145139412207989] | -0.010181119998076138 | None | () |
| edpf | adaptive | 5 | [0.8766945550534653, 0.9991462976276059] | 0.5860498454824892 | None | () |
| edpf | classic | 5 | [0.8674704128914204, 0.9936991688265261] | 0.7105391008586759 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.6502427042517106, 0.9956536726466138] | 0.5618179803934031 | None | () |
| edpf | rtt-threshold | 5 | [0.8721163118463024, 0.9909420828747363] | 0.7003579808605997 | None | () |
| enhanced | adaptive | 5 | [0.9872488926669949, 1.5365744069015095] | 0.024231865089086035 | None | () |
| enhanced | classic | 5 | [0.9768615529911753, 1.5281973366699437] | 0.14872112046527275 | None | () |
| enhanced | edpf | 5 | [1.0043653003778241, 1.5378873049422135] | -0.5618179803934031 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.9820933165195462, 1.5124719896214176] | 0.1385400004671966 | None | () |
| rtt-threshold | adaptive | 5 | [0.9916009086561453, 1.0159357776180207] | -0.11430813537811058 | None | () |
| rtt-threshold | classic | 5 | [0.985693699582547, 1.0103971162153307] | 0.010181119998076138 | None | () |
| rtt-threshold | edpf | 5 | [1.0091407129455907, 1.1466360466105296] | -0.7003579808605997 | None | () |
| rtt-threshold | enhanced | 5 | [0.6611692691580405, 1.0182331792501282] | -0.1385400004671966 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--F@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 12803264.56888889 | 12803217.777777778 | 782.9650452749423 | [12802281.955555556, 12804153.6] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.008982608569206097 | 0.006075094747146931 | 0.011773904483432409 | [0.00022504277272156206, 0.029225759971033244] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 30.96433264971584 | 31.522158077484967 | 1.8291458230430575 | [27.909650490640544, 32.63187847366346] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2254.2 | 2261.0 | 31.332092173999488 | [2202.0, 2281.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 50.09288761484312 | 50.243327926046085 | 0.6962279230400114 | [48.93333333333333, 50.68888888888889] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 4.955510420756083 | 5.044332348170041 | 0.2926861805519223 | [4.466666666666667, 5.222222222222222] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.0051085563779368565 | 0.003465501668233516 | 0.003724564693610422 | [0.0029884266386539403, 0.01171973150507915] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.245686107744106 | 13.246119999999996 | 0.0008816632544283719 | [13.244539999999995, 13.246525925925926] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1989.4 | 1990.0 | 1.8165902124584952 | [1987.0, 1991.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 61.717099999999995 | 61.6905 | 0.1375965115836876 | [61.59, 61.913] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 68.6 | 66.0 | 9.6591925128346 | [57.0, 80.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 68.6 | 66.0 | 9.6591925128346 | [57.0, 80.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.00421429487650854 | 0.004287089683736308 | 0.0003675458953789615 | [0.003722872968310179, 0.004587410517748469] | 0.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 12803030.613333333 | 12803451.733333332 | 1624.2649830645428 | [12800644.266666668, 12804855.466666669] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.018533865758083256 | 0.013901703183558878 | 0.009018990745765423 | [0.010272614633840216, 0.0313051232403668] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 35.6302723629298 | 34.85489725748338 | 2.8234938075345988 | [32.63724614749946, 40.26544760293404] | 0.0 |
| classic | switch_count | 5 | 0 | 20480.8 | 15461.0 | 9774.620028420542 | [12357.0, 31600.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 455.1258683140374 | 343.5777777777778 | 217.2098758375903 | [274.6, 702.2222222222222] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 5.702197432649645 | 5.57765382991489 | 0.452327655551455 | [5.222222222222222, 6.444444444444445] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.08396457636732339 | 0.07815795640965549 | 0.03786844856559993 | [0.03672405346770243, 0.1258283407758196] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.242681925925925 | 13.247065454545451 | 0.006687844681197537 | [13.233314545454546, 13.247714545454548] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1992.4 | 1992.0 | 0.5477225575051661 | [1992.0, 1993.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 61.8042 | 62.086 | 0.6199509658029414 | [61.098, 62.499] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 56.4 | 70.0 | 20.032473636573194 | [33.0, 72.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 56.4 | 70.0 | 20.032473636573194 | [33.0, 72.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.004143976613176888 | 0.004322792742067312 | 0.0003935886106112783 | [0.0035907307322129265, 0.004557918248016125] | 0.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 12803217.777777778 | 12803685.688888889 | 1097.3488248759397 | [12802048.0, 12804387.555555556] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.014264009872911664 | 0.012750623495434934 | 0.009092379454522426 | [0.004413019205662616, 0.02612148707155501] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 34.65790907873266 | 34.71666755541335 | 1.4302569377886332 | [32.77253417231021, 36.516595584729835] | 0.0 |
| edpf | switch_count | 5 | 0 | 8112.8 | 8259.0 | 1706.904420288377 | [5626.0, 10191.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 180.28233199756056 | 183.52925490544655 | 37.93246435531069 | [125.01944401235528, 226.46666666666667] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 5.546590816500374 | 5.555432101508855 | 0.22905543246841784 | [5.2444444444444445, 5.844314570787316] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.03154465982150593 | 0.032240485189417245 | 0.00865799027150646 | [0.02017986002828444, 0.04138004396851287] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.244512693602696 | 13.243961818181818 | 0.00254472798305938 | [13.241209259259264, 13.247520370370372] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1991.8 | 1992.0 | 0.8366600265340756 | [1991.0, 1993.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 63.398900000000005 | 63.5035 | 0.23751668362453957 | [62.999, 63.607] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 64.0 | 62.0 | 7.713624310270756 | [55.0, 72.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 64.0 | 62.0 | 7.713624310270756 | [55.0, 72.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.004180688726148181 | 0.004268846503178928 | 0.00024828402141577295 | [0.0038509745508710103, 0.004440250874174391] | 0.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 12803732.48 | 12804153.6 | 1038.4042945372794 | [12802048.0, 12804621.51111111] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.006129418668329822 | 0.007244917231921694 | 0.0036184262422267527 | [0.0008156448503089386, 0.010352092661220724] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 32.0184518279456 | 32.767144839277435 | 1.8979862865392556 | [28.88110061875536, 33.605734193640124] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2264.0 | 2260.0 | 16.777961735562517 | [2241.0, 2281.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 50.310666775306224 | 50.22222222222222 | 0.37332272928925986 | [49.79889335792538, 50.68888888888889] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 5.1243972356169865 | 5.2443279038243595 | 0.3036853440828557 | [4.622222222222222, 5.377658274260572] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.003386689886397914 | 0.003447258784057811 | 0.00032264288307625865 | [0.0030071581433104566, 0.003851327694547331] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.24397142087542 | 13.244192592592588 | 0.0038634009795575747 | [13.238329090909092, 13.24877592592592] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1988.2 | 1988.0 | 1.3038404810405297 | [1987.0, 1990.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 61.6216 | 61.6105 | 0.0367379231857206 | [61.596999999999994, 61.686499999999995] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 67.0 | 64.0 | 7.280109889280518 | [59.0, 78.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 67.0 | 64.0 | 7.280109889280518 | [59.0, 78.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.00437956630099083 | 0.004288881619598008 | 0.0002853423400799573 | [0.0041450010177457855, 0.004846556540076583] | 0.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 12804013.226666667 | 12804387.555555556 | 803.6637252657459 | [12802749.866666667, 12804621.51111111] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.010471324455329833 | 0.010523151202356695 | 0.006954853305350242 | [0.00025276270599225437, 0.01963792235905948] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 28.935506691256798 | 32.210592408885255 | 7.464908052752424 | [15.68877992329325, 33.46258685624023] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 2286.2 | 2281.0 | 12.070625501605127 | [2273.0, 2303.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 50.80421995560592 | 50.68888888888889 | 0.2685432821881635 | [50.50998866691851, 51.17777777777778] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 4.631088198040043 | 5.155440990200217 | 1.194687659188914 | [2.511111111111111, 5.355555555555555] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.00427836087528335 | 0.0035944037897918933 | 0.0017999163761401062 | [0.003312636895172239, 0.007490224155972903] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.245377373737375 | 13.24518148148148 | 0.0017398680571738406 | [13.243007272727272, 13.247101818181818] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1989.2 | 1989.0 | 0.4472135954999579 | [1989.0, 1990.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 61.5981 | 61.6075 | 0.09041252678694654 | [61.498, 61.715] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 70.6 | 74.0 | 7.127411872482185 | [62.0, 78.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 70.6 | 74.0 | 7.127411872482185 | [62.0, 78.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.004377567612841811 | 0.004272792852782864 | 0.0002336184953721138 | [0.004160459285636423, 0.0046676353069378855] | 0.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9998355623766717, 1.0002558760097962] | 0.0 | None | () |
| adaptive | edpf | 5 | [0.9998720947229939, 1.0001644736842106] | 0.0 | None | () |
| adaptive | enhanced | 5 | [0.99987209939704, 1.0001644736842106] | 0.0 | None | () |
| adaptive | rtt-threshold | 5 | [0.999817288191336, 1.0000913692597262] | 0.0 | None | () |
| classic | adaptive | 5 | [0.9997441894459876, 1.0001644646675074] | 0.0 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.999890350877193, 1.0001461988304092] | 0.0 | None | () |
| classic | enhanced | 5 | [0.999689389925271, 1.0001461988304092] | 0.0 | None | () |
| classic | rtt-threshold | 5 | [0.999817288191336, 1.0000365430294174] | 0.0 | None | () |
| edpf | adaptive | 5 | [0.9998355533629337, 1.0001279216388588] | 0.0 | None | () |
| edpf | classic | 5 | [0.9998538225405643, 1.0001096611470555] | 0.0 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.9997990170104695, 1.0000365457003983] | 0.0 | None | () |
| edpf | rtt-threshold | 5 | [0.9998720923858425, 1.0] | 0.0 | None | () |
| enhanced | adaptive | 5 | [0.9998355533629337, 1.0001279169636168] | 0.0 | None | () |
| enhanced | classic | 5 | [0.9998538225405643, 1.000310706583324] | 0.0 | None | () |
| enhanced | edpf | 5 | [0.9999634556351411, 1.0002010233918128] | 0.0 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.9998720923858425, 1.000146190815562] | 0.0 | None | () |
| rtt-threshold | adaptive | 5 | [0.9999086390878528, 1.0001827451983698] | 0.0 | None | () |
| rtt-threshold | classic | 5 | [0.9999634583059269, 1.0001827451983698] | 0.0 | None | () |
| rtt-threshold | edpf | 5 | [1.0, 1.000127923976608] | 0.0 | None | () |
| rtt-threshold | enhanced | 5 | [0.9998538305530688, 1.000127923976608] | 0.0 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 12803404.942222223 | 12803217.777777778 | 533.5007500193908 | [12802749.866666667, 12804153.6] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.2396005870663868 | 0.23401262171954634 | 0.010375528798633555 | [0.23009035485019236, 0.2519185133815959] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 50.54189874654697 | 54.01419920825459 | 8.685059493242292 | [35.13134150568875, 56.09290896215289] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2203.8 | 2215.0 | 28.59545418418809 | [2153.0, 2222.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 48.97268238483589 | 49.22222222222222 | 0.6356356690249836 | [47.84338125819426, 49.376680518210705] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 8.088772842084989 | 8.644252349947779 | 1.390112531146863 | [5.622222222222222, 8.97757827603831] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.06755038139346857 | 0.05443399941781819 | 0.030696089905646394 | [0.05225185083630381, 0.12239624615579722] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.29347319191919 | 13.29302777777778 | 0.01832249772390677 | [13.269429090909089, 13.316656363636362] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1990.6 | 1991.0 | 1.140175425099138 | [1989.0, 1992.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 62.1677 | 62.0705 | 0.3137936423830148 | [61.842, 62.678] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 235.2 | 237.0 | 58.8362133383854 | [164.0, 314.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 235.2 | 237.0 | 58.8362133383854 | [164.0, 314.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.00686490260692074 | 0.007115600862720519 | 0.001388006036903647 | [0.004967727899049967, 0.00827297194776106] | 0.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 12804481.137777777 | 12805323.377777778 | 1975.5049736309227 | [12801346.133333333, 12806493.155555556] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.21394209967215652 | 0.21661374307854428 | 0.009012520535538259 | [0.19880069038078926, 0.2207544113541953] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 59.31257305537072 | 59.85481637958853 | 2.180650082249087 | [56.36544712571205, 62.051855805813176] | 0.0 |
| classic | switch_count | 5 | 0 | 46176.6 | 46077.0 | 1204.9650617341567 | [44616.0, 47714.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 1026.1327674199833 | 1023.910579764894 | 26.766275695867876 | [991.4666666666668, 1060.287549165574] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 9.493205336177716 | 9.577564943001269 | 0.3489852807042403 | [9.022222222222222, 9.933112597497834] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.1524081667787632 | 0.1515338464148557 | 0.018060024609290463 | [0.1276808059049007, 0.16986024634032354] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.241780727272726 | 13.239441818181822 | 0.006869009167423334 | [13.233261818181818, 13.249216363636362] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1594.6 | 1993.0 | 891.408884855878 | [0.0, 1994.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 60.84740000000001 | 60.847 | 0.013011533345457401 | [60.834, 60.861] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 35.6 | 33.0 | 8.294576541331088 | [27.0, 49.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 35.6 | 33.0 | 8.294576541331088 | [27.0, 49.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.002935965020276994 | 0.0027604242336190616 | 0.000331161203246303 | [0.0026349742862854133, 0.003453791899949102] | 0.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 4471311.786666667 | 4558624.0 | 244319.81018991873 | [4102176.711111111, 4733154.844444444] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.6333643603009985 | 0.6366104010498121 | 0.023495742640934847 | [0.6047227926078029, 0.6680480729382512] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.20126801523977772 | 0.205609659546916 | 0.01763286086183726 | [0.18261308196598494, 0.21804066688198082] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 271.13147670506726 | 292.4859197278243 | 41.25065877939321 | [199.81974155945636, 299.212813831497] | 0.0 |
| edpf | switch_count | 5 | 0 | 75.0 | 86.0 | 37.5632799419859 | [16.0, 108.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 1.6666549138414208 | 1.9111111111111112 | 0.8347331786051455 | [0.3555476544965667, 2.4] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 15.084322669372778 | 15.57743161263083 | 1.9335114707720955 | [11.821959512010844, 16.666666666666668] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.6883805735243376 | 0.6835141301992377 | 0.009547856710776062 | [0.6811167497669739, 0.7045156208978735] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 6.181601360818713 | 6.176882777777777 | 0.15863815451388957 | [5.931271052631579, 6.318848000000001] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1801.8 | 1818.0 | 60.36306817914411 | [1737.0, 1867.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 793.4414 | 796.829 | 13.57062667307589 | [774.7850000000001, 808.313] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 2056.8 | 1895.0 | 561.9058640021476 | [1319.0, 2740.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 2056.8 | 1895.0 | 561.9058640021476 | [1319.0, 2740.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 33592.0 | 33705.0 | 3308.4327860786293 | [28272.0, 37076.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.29863517750905355 | 0.29494907503637496 | 0.010914541657241874 | [0.2881386050644158, 0.3142021992298056] | 0.0 |
| edpf | load[0].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 12802656.284444446 | 12802749.866666667 | 632.1131025532691 | [12801814.044444444, 12803451.733333332] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.2360151785643049 | 0.23332094365229478 | 0.007384208362035507 | [0.2274059714735361, 0.24631108396892304] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 53.294515864953304 | 53.457806433753916 | 2.0565443359697175 | [49.989260640505734, 55.26994479837885] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2194.6 | 2195.0 | 18.420097719610503 | [2167.0, 2218.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 48.76802164890163 | 48.77669385124775 | 0.40928849640183357 | [48.15448545587875, 49.28779360458657] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 8.528734917001845 | 8.555365436323637 | 0.32893023398749255 | [8.0, 8.844247905602098] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.0766407342895182 | 0.06917667642407725 | 0.02226120790307489 | [0.052995431133318505, 0.10805405928458536] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.302300363636363 | 13.304381818181817 | 0.010557098618449067 | [13.29090727272727, 13.312554545454535] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1990.8 | 1991.0 | 0.8366600265340756 | [1990.0, 1992.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 62.37539999999999 | 62.374 | 0.33539200944566455 | [61.988, 62.814] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 276.0 | 289.0 | 42.44997055358225 | [231.0, 329.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 276.0 | 289.0 | 42.44997055358225 | [231.0, 329.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.007887554044377344 | 0.008223386951021146 | 0.0009561098439874211 | [0.006876956765658649, 0.009107007336200354] | 0.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 10567117.368888889 | 10569176.177777778 | 1413711.9574256812 | [9071158.755555555, 12804387.555555556] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.16677440619974992 | 0.16834409418913787 | 0.10413627489896796 | [0.0, 0.265590156100669] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.2563997048655733 | 0.2569714463280069 | 0.013990112986662915 | [0.2384434301298229, 0.2729918760638419] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 81.68149842316012 | 87.2978793377122 | 28.373425015507276 | [35.404530780282535, 104.45805007824751] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 1490.0 | 1416.0 | 445.45482374759393 | [1166.0, 2256.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 33.110853141535124 | 31.466666666666665 | 9.89912130365501 | [25.91111111111111, 50.13333333333333] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 10.395451656629854 | 11.533077042732383 | 2.775956618467607 | [5.666666666666667, 12.644444444444444] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.2825377094705958 | 0.3209094684385382 | 0.1378652278207615 | [0.044555056452591967, 0.3846727925742805] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.126361023647851 | 11.957897826086953 | 0.6746274684807059 | [11.61033794871795, 13.254725454545454] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1609.8 | 1540.0 | 224.82259672906545 | [1396.0, 1990.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 62.4529 | 62.265 | 0.4823961546281242 | [61.943, 63.1565] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 1409.2 | 1747.0 | 842.1251094700834 | [103.0, 2106.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 1409.2 | 1747.0 | 842.1251094700834 | [103.0, 2106.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 8622.6 | 8865.0 | 5358.865019759315 | [0.0, 13339.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.08286031549796712 | 0.09124071551004263 | 0.04594255211459004 | [0.004047921582864404, 0.12037598544572468] | 0.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 0.8 | — |
| edpf | load[0] graded=True | 0.0 | 0.0 | 1.0 |
| edpf | settled_rate | — | 0.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | load[0] graded=True | 0.2 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9997077038309067, 1.0001462068461355] | 0.0 | None | () |
| adaptive | edpf | 5 | [2.704908309030696, 3.1213071746321432] | -63.66104010498121 | None | () |
| adaptive | enhanced | 5 | [0.9999817271498009, 1.0000913675900884] | 0.0 | None | () |
| adaptive | rtt-threshold | 5 | [0.99987209939704, 1.4115234828359942] | -16.834409418913786 | None | () |
| classic | adaptive | 5 | [0.9998538145271814, 1.000292381631124] | 0.0 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [2.705699174534131, 3.121592334892209] | -63.66104010498121 | None | () |
| classic | enhanced | 5 | [0.9998355443482074, 1.0003655037555512] | 0.0 | None | () |
| classic | rtt-threshold | 5 | [1.000164443632377, 1.411652438552601] | -16.834409418913786 | None | () |
| edpf | adaptive | 5 | [0.3203785927022237, 0.36969829870438387] | 63.66104010498121 | None | () |
| edpf | classic | 5 | [0.32034932583037967, 0.36959023730795226] | 63.66104010498121 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.32040786492215484, 0.36972532392770335] | 63.66104010498121 | None | () |
| edpf | rtt-threshold | 5 | [0.3696510140690663, 0.47337092731829583] | 46.826630686067425 | None | () |
| enhanced | adaptive | 5 | [0.9999086407571854, 1.0000182731841023] | 0.0 | None | () |
| enhanced | classic | 5 | [0.9996346297886333, 1.0001644827019025] | 0.0 | None | () |
| enhanced | edpf | 5 | [2.7047105926548367, 3.121022014372077] | -63.66104010498121 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.9997990133382056, 1.4113945271193873] | -16.834409418913786 | None | () |
| rtt-threshold | adaptive | 5 | [0.7084543843300627, 1.0001279169636168] | 16.834409418913786 | None | () |
| rtt-threshold | classic | 5 | [0.7083896663865239, 0.999835583404885] | 16.834409418913786 | None | () |
| rtt-threshold | edpf | 5 | [2.1125082726671076, 2.7052543126884485] | -46.826630686067425 | None | () |
| rtt-threshold | enhanced | 5 | [0.7085191140998465, 1.0002010270655532] | 16.834409418913786 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@baseline--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| upstream-classic | useful_goodput_bps | 5 | 0 | 12801720.462222222 | 12801814.044444444 | 853.2163771703806 | [12800410.311111111, 12802749.866666667] | 0.0 |
| upstream-classic | viewer_loss_ratio | 5 | 0 | 0.00010547167252233975 | 9.09338910611985e-05 | 7.205179920596968e-05 | [0.0, 0.00018184793875361424] | 0.0 |
| upstream-classic | per_link_share_gini | 5 | 0 | 0.19352791800935867 | 0.19318863712342915 | 0.0013222102641961705 | [0.1923385543949721, 0.19580386649123122] | 0.0 |
| upstream-classic | cpu_ms_per_mb | 5 | 0 | 44.827229226614506 | 46.7980678646972 | 4.566302121430841 | [37.494686169754495, 48.32295195248887] | 0.0 |
| upstream-classic | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | sender_cpu_percent | 5 | 0 | 7.173241582285826 | 7.488888888888889 | 0.73090593130694 | [5.999866669629564, 7.733333333333333] | 0.0 |
| upstream-classic | diagnostics.loss_ratio | 5 | 0 | 0.36246104620073316 | 0.3633202159787682 | 0.0021960963183303758 | [0.35900384022811416, 0.3643358407485552] | 0.0 |
| upstream-classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.383352363636362 | 13.387654545454543 | 0.027489368956377772 | [13.336581818181816, 13.40747090909091] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1989.0 | 1988.0 | 2.345207879911715 | [1987.0, 1993.0] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-classic | diagnostics.ms_rtt_median | 5 | 0 | 66.52799999999999 | 66.802 | 0.41174506675854805 | [66.027, 66.875] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_delta | 5 | 0 | 659.8 | 677.0 | 114.21777444863825 | [468.0, 765.0] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_sum | 5 | 0 | 659.8 | 677.0 | 114.21777444863825 | [468.0, 765.0] | 0.0 |
| upstream-classic | diagnostics.pkt_drop_delta | 5 | 0 | 5.8 | 5.0 | 3.96232255123179 | [0.0, 10.0] | 0.0 |
| upstream-classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | diagnostics.retrans_ratio | 5 | 0 | 0.013932956171168012 | 0.01417636912462268 | 0.001756955666375947 | [0.010967800126274015, 0.01558744394618834] | 0.0 |
| upstream-classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| upstream-enhanced | useful_goodput_bps | 5 | 0 | 12674822.96888889 | 12790350.222222222 | 168794.95296146232 | [12449243.022222225, 12800410.311111111] | 0.0 |
| upstream-enhanced | viewer_loss_ratio | 5 | 0 | 0.010032273538670752 | 0.0009075727873375444 | 0.013022881178636326 | [0.00043630810624102384, 0.02689090909090909] | 0.0 |
| upstream-enhanced | per_link_share_gini | 5 | 0 | 0.2147967853500054 | 0.21164699324039465 | 0.017871721836534064 | [0.19134100705364607, 0.24081855919122952] | 0.0 |
| upstream-enhanced | cpu_ms_per_mb | 5 | 0 | 41.9781555603728 | 40.13017946613305 | 8.69062339820209 | [34.03539603392337, 55.69281056653124] | 0.0 |
| upstream-enhanced | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | sender_cpu_percent | 5 | 0 | 6.639944198770893 | 6.288888888888889 | 1.3025655763437103 | [5.444323459478678, 8.666666666666666] | 0.0 |
| upstream-enhanced | diagnostics.loss_ratio | 5 | 0 | 0.27918656320373636 | 0.26984333017484197 | 0.026741232006167705 | [0.2549798186297636, 0.3237159425082376] | 0.0 |
| upstream-enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.70730674442539 | 13.663974545454549 | 0.18804517777664795 | [13.528201818181817, 13.938805660377366] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1855.6 | 1982.0 | 178.61494898244098 | [1655.0, 1990.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rtt_median | 5 | 0 | 66.25380000000001 | 66.706 | 1.3121210691091003 | [64.151, 67.427] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 2202.4 | 1674.0 | 1143.4519229071243 | [1228.0, 3692.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 2202.4 | 1674.0 | 1143.4519229071243 | [1228.0, 3692.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 544.0 | 50.0 | 707.9028888202109 | [24.0, 1479.0] | 0.0 |
| upstream-enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.05074525252790842 | 0.03287541116251253 | 0.03438412730513706 | [0.023970170454545456, 0.10153899965022736] | 0.0 |
| upstream-enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| upstream-classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-classic | settled_rate | — | 1.0 | — |
| upstream-classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| upstream-enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-enhanced | settled_rate | — | 1.0 | — |
| upstream-enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| upstream-classic | upstream-classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| upstream-classic | upstream-enhanced | 5 | [1.0001279403432457, 1.0283018867924527] | -0.08166388962763459 | None | () |
| upstream-enhanced | upstream-classic | 5 | [0.9724770642201837, 0.9998720760233918] | 0.08166388962763459 | None | () |
| upstream-enhanced | upstream-enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--H@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 6986404.195555556 | 6996557.866666666 | 101124.07177968706 | [6837351.111111111, 7083823.288888888] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.4186833687258785 | 0.4194340238941294 | 0.009438093139521606 | [0.4086442422163482, 0.43278501416685855] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.07623346849043947 | 0.09003486870478765 | 0.032129635958139245 | [0.020823155742871673, 0.10212279317799779] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 94.05254037686775 | 86.51873462196191 | 30.979998585355467 | [59.70922237222855, 139.16530620950732] | 0.0 |
| adaptive | switch_count | 5 | 0 | 927.2 | 927.0 | 18.08867048735202 | [898.0, 945.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 10.302222222222222 | 10.3 | 0.2009852276372447 | [9.977777777777778, 10.5] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 8.191111111111113 | 7.566666666666666 | 2.6225800132552535 | [5.277777777777778, 12.077777777777778] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.5548795007260547 | 0.5543415917071192 | 0.0062810330236233135 | [0.5478837065239331, 0.5646373637313953] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.546352334445071 | 8.543249166666666 | 0.053211548521633685 | [8.481636724137932, 8.619713114754099] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1304.0 | 1291.0 | 53.53970489272424 | [1252.0, 1381.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 694.4997 | 692.891 | 54.47808878416165 | [606.7835, 744.2574999999999] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 1111.0 | 1197.0 | 338.60227406206235 | [625.0, 1498.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 1111.0 | 1197.0 | 338.60227406206235 | [625.0, 1498.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 42071.0 | 42375.0 | 1060.1964440611937 | [40504.0, 43227.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.18446613534616502 | 0.18693348097761608 | 0.0057011067781042825 | [0.17475744013589886, 0.18846603854917177] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].failover_ms | 5 | 0 | 5112.4 | 5237.0 | 1926.5227224198525 | [2853.0, 7515.0] | 0.0 |
| adaptive | episode[3].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | episode[4].failover_ms | 5 | 0 | 4139.0 | 3960.0 | 1896.4795016029043 | [1943.0, 6887.0] | 0.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | 2897.6 | 2722.0 | 1191.8583388977065 | [1973.0, 4884.0] | 0.0 |
| adaptive | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].failover_ms | 5 | 0 | 4572.2 | 2989.0 | 3778.76377933313 | [1946.0, 10960.0] | 0.0 |
| adaptive | episode[6].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 7038365.724444444 | 7108622.577777778 | 151778.7365377701 | [6827524.977777778, 7201736.888888889] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.41311725099062063 | 0.40971568442854456 | 0.013577417416204534 | [0.3975785047565085, 0.43105570013516514] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.06777395091577271 | 0.06627179613762184 | 0.02378066292315664 | [0.036571240708403306, 0.09584111686269008] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 92.73266912875287 | 91.56365692804106 | 14.061742238062882 | [77.01845752271612, 108.9131141442948] | 0.0 |
| classic | switch_count | 5 | 0 | 15171.4 | 15234.0 | 613.5872391111145 | [14461.0, 15855.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 168.57074965833712 | 169.26666666666668 | 6.81802806036912 | [160.67777777777778, 176.16666666666666] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 8.157756271843894 | 8.144444444444444 | 1.2446447292243623 | [6.933333333333334, 9.677670248108354] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.5553016537826186 | 0.55334965161398 | 0.010269743870258474 | [0.5456100182886819, 0.5681032941092313] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.578763058854872 | 8.575785901639344 | 0.15644088791578026 | [8.390089482758619, 8.734588548387094] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1408.8 | 1430.0 | 55.63002786265705 | [1345.0, 1465.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 730.5473 | 730.085 | 35.42160525300908 | [695.766, 782.2975] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 791.2 | 681.0 | 455.75618481815474 | [415.0, 1554.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 791.2 | 681.0 | 455.75618481815474 | [415.0, 1554.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 41341.2 | 41344.0 | 1141.9506118917752 | [39996.0, 42734.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.17922015082496812 | 0.18086282324734165 | 0.0175282360476719 | [0.15249184292031734, 0.1972574802909932] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].failover_ms | 5 | 0 | 3364.2 | 3372.0 | 679.388843005241 | [2480.0, 4376.0] | 0.0 |
| classic | episode[3].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| classic | episode[4].failover_ms | 5 | 0 | 3482.4 | 2526.0 | 2116.641041839641 | [1974.0, 6980.0] | 0.0 |
| classic | episode[4].recovery_ms | 5 | 0 | 2241.8 | 1979.0 | 438.3807705636733 | [1963.0, 2977.0] | 0.0 |
| classic | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].failover_ms | 5 | 0 | 6580.4 | 5984.0 | 3346.666296480723 | [2989.0, 11951.0] | 0.0 |
| classic | episode[6].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 6062139.377777778 | 5760102.755555555 | 610328.2513587819 | [5564048.0, 6953744.0] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.49595531436666623 | 0.5240467808341787 | 0.05093955833761783 | [0.42012131437294126, 0.5349594305341232] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.0428773736705297 | 0.044867475658458354 | 0.02008058414935771 | [0.012740222603482473, 0.06845986080899598] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 109.0247537538905 | 103.84126409006137 | 28.45835414441418 | [78.77020900531521, 153.39232528506125] | 0.0 |
| edpf | switch_count | 5 | 0 | 1587.0 | 403.0 | 2033.7370774020912 | [31.0, 4723.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 17.63325982797722 | 4.477728025244164 | 22.597047324397543 | [0.34444444444444444, 52.477777777777774] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 8.271070025147868 | 7.444361729314119 | 2.2671486041619118 | [5.511111111111111, 11.044321729758558] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.6194238739141062 | 0.6379582598626208 | 0.038077174671301194 | [0.5629394828636288, 0.6508646033782339] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 7.322809406841347 | 7.00530081632653 | 0.9453092530760306 | [6.484697446808509, 8.58177152542373] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1602.0 | 1679.0 | 276.9945847846127 | [1141.0, 1881.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 789.1658 | 832.274 | 104.24208734815319 | [603.121, 845.525] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 1620.4 | 1790.0 | 881.7104399971682 | [484.0, 2476.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 1620.4 | 1790.0 | 881.7104399971682 | [484.0, 2476.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 49816.2 | 52695.0 | 5313.862879299766 | [41834.0, 53937.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.25182932440678335 | 0.2612864333148336 | 0.03184127587556056 | [0.19908155186064924, 0.27941336822592955] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | 5952.0 | None | [2965.0, +inf] | 0.4 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | 2948.0 | None | [1876.0, +inf] | 0.4 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].failover_ms | 5 | 0 | 5013.6 | 6162.0 | 1872.9198594707675 | [2440.0, 6486.0] | 0.0 |
| edpf | episode[3].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| edpf | episode[4].failover_ms | 5 | 0 | 4954.8 | 3936.0 | 3472.7603862057626 | [1977.0, 10978.0] | 0.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | 2342.0 | 1976.0 | 862.8473213726749 | [1929.0, 3885.0] | 0.0 |
| edpf | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].failover_ms | 5 | 0 | 2772.4 | 2984.0 | 830.6577514235331 | [1966.0, 3947.0] | 0.0 |
| edpf | episode[6].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| edpf | load[0].reached_ms | 5 | 0 | +inf | 1000.0 | None | [1000.0, +inf] | 0.2 |
| enhanced | useful_goodput_bps | 5 | 0 | 6946234.026666666 | 6994569.244444445 | 79351.87361555404 | [6846124.444444444, 7009893.333333333] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.4222346108674538 | 0.4187808438173351 | 0.00644277372681764 | [0.4163692908833556, 0.430440466613033] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.07173730943749641 | 0.07314102885311295 | 0.019471445748244862 | [0.04075362175976608, 0.08914837293659854] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 94.11681593429918 | 99.19643100589596 | 33.194710208791946 | [57.06220922049218, 142.75359477861141] | 0.0 |
| enhanced | switch_count | 5 | 0 | 914.8 | 911.0 | 11.388590782006348 | [904.0, 933.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 10.16439945728998 | 10.122222222222222 | 0.12655840905905416 | [10.044332840746216, 10.366666666666667] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 8.153294963389294 | 8.488888888888889 | 2.8125301421990865 | [4.999944445061722, 12.266530371884755] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.558997785069612 | 0.5582683656076228 | 0.004529909592130705 | [0.5532322471142537, 0.5648722490337108] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.473819875628287 | 8.4837365 | 0.0681128319444042 | [8.384624310344826, 8.569066999999997] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1340.8 | 1353.0 | 77.78945944020951 | [1217.0, 1418.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 678.2978 | 678.617 | 83.43586930136823 | [569.966, 762.1475] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 760.8 | 706.0 | 269.76508298888496 | [416.0, 1080.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 760.8 | 706.0 | 269.76508298888496 | [416.0, 1080.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 42383.6 | 42175.0 | 552.5900831538692 | [41771.0, 43107.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.1784153372743924 | 0.1810372614749871 | 0.01401561750538004 | [0.1547322797013794, 0.1918911676151142] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 5 | 0 | 5283.8 | 3845.0 | 2888.6536137100275 | [2325.0, 8511.0] | 0.0 |
| enhanced | episode[3].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | episode[4].failover_ms | 5 | 0 | 4915.2 | 1923.0 | 5690.783838804633 | [1886.0, 14979.0] | 0.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | 2719.4 | 1927.0 | 1308.8912483472413 | [1892.0, 4917.0] | 0.0 |
| enhanced | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].failover_ms | 5 | 0 | 3377.2 | 3965.0 | 902.5215786893962 | [1956.0, 3987.0] | 0.0 |
| enhanced | episode[6].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 6941742.08 | 6982637.511111111 | 134504.0633372408 | [6710313.244444445, 7059491.911111111] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.422054106807397 | 0.4179381319930593 | 0.012259306154150643 | [0.4113529173299584, 0.4431014298436078] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.047904306259422 | 0.04305691546152213 | 0.02927625969721542 | [0.022116517102696354, 0.09637547691955768] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 97.9514228057219 | 100.47937475739519 | 16.34723029514523 | [76.36394389859967, 115.46098747634005] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 833.8 | 835.0 | 10.756393447619885 | [817.0, 844.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 9.26440281527736 | 9.27777777777778 | 0.11947097526175571 | [9.077777777777778, 9.37767358140465] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 8.491074642380518 | 8.866568149242786 | 1.3674284943129629 | [6.677777777777778, 10.077777777777778] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.5561283989590082 | 0.5535366599422213 | 0.0073449916606991485 | [0.5486406516517562, 0.5680297397769517] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.426411566635469 | 8.452523166666667 | 0.13250474624842054 | [8.210445263157895, 8.572633934426234] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1465.8 | 1459.0 | 56.698324490235166 | [1414.0, 1547.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 703.0383 | 763.1995 | 103.11172861488646 | [590.6600000000001, 793.426] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 1088.0 | 1115.0 | 101.84547118060773 | [928.0, 1202.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 1088.0 | 1115.0 | 101.84547118060773 | [928.0, 1202.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 42298.0 | 41886.0 | 1116.7575833635517 | [41603.0, 44284.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.19861634308325113 | 0.19834110119282725 | 0.011635914540199967 | [0.1801602012445386, 0.2097197271990207] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | 3435.8 | 3190.0 | 1206.046516515843 | [2315.0, 5174.0] | 0.0 |
| rtt-threshold | episode[3].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | 3973.8 | 3980.0 | 2004.7620058251305 | [1959.0, 5977.0] | 0.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | 1964.8 | 1975.0 | 19.94241710525582 | [1930.0, 1977.0] | 0.0 |
| rtt-threshold | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].failover_ms | 5 | 0 | 4182.8 | 3977.0 | 1790.8579508157536 | [1991.0, 5989.0] | 0.0 |
| rtt-threshold | episode[6].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[2] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[3] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[4] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[5] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[6] graded=True | — | 1.0 | 0.0 |
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 0.0 | 1.0 |
| classic | episode[2] graded=True | — | 0.0 | 1.0 |
| classic | episode[3] graded=True | — | 1.0 | 0.0 |
| classic | episode[4] graded=True | — | 1.0 | 0.0 |
| classic | episode[5] graded=True | — | 0.0 | 1.0 |
| classic | episode[6] graded=True | — | 1.0 | 0.0 |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 0.6 | 0.4 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | episode[3] graded=True | — | 1.0 | 0.0 |
| edpf | episode[4] graded=True | — | 1.0 | 0.0 |
| edpf | episode[5] graded=True | — | 0.0 | 1.0 |
| edpf | episode[6] graded=True | — | 1.0 | 0.0 |
| edpf | load[0] graded=True | 0.0 | 0.8 | 0.19999999999999996 |
| edpf | settled_rate | — | 0.8 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[4] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[5] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[6] graded=True | — | 1.0 | 0.0 |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[3] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[4] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[5] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[6] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [0.9715097864046129, 1.0192038576318052] | 0.9718339465584869 | 1.450320295453205 | () |
| adaptive | edpf | 5 | [1.0187063672302128, 1.2633756896839994] | -10.461275694004923 | 1.5173306641077398 | () |
| adaptive | enhanced | 5 | [0.9987184963690731, 1.0110695562159815] | 0.06531800767943241 | 1.2277545916894175 | () |
| adaptive | rtt-threshold | 5 | [0.9685330328588709, 1.0426574158008506] | 0.14958919010701344 | 1.5667772189924642 | () |
| classic | adaptive | 5 | [0.9811579818031431, 1.0293257093176842] | -0.9718339465584869 | 1.1009406636886259 | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| classic | edpf | 5 | [1.0222726890402893, 1.2943340691685064] | -11.43310964056341 | 1.4545189818809319 | () |
| classic | enhanced | 5 | [0.9897538589904048, 1.0351393663002417] | -0.9065159388790545 | 1.5228831403150016 | () |
| classic | rtt-threshold | 5 | [0.9671411290990738, 1.073234084094554] | -0.8222447564514734 | 1.4806196102752816 | () |
| edpf | adaptive | 5 | [0.7915301902398677, 0.9816371352609938] | 10.461275694004923 | 0.7540828935942823 | () |
| edpf | classic | 5 | [0.7725980670835702, 0.978212575490793] | 11.43310964056341 | 0.9842259649035614 | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| edpf | enhanced | 5 | [0.7954811519550457, 0.9925034227134605] | 10.526593701684355 | 0.9949836970152997 | () |
| edpf | rtt-threshold | 5 | [0.8041275229820345, 0.99399705705304] | 10.610864884111937 | 0.9947338936143821 | () |
| enhanced | adaptive | 5 | [0.9890516373003947, 1.0012831479897348] | -0.06531800767943241 | 1.4219514266289262 | () |
| enhanced | classic | 5 | [0.9660534924627247, 1.0103522112255736] | 0.9065159388790545 | 1.3677519183241778 | () |
| enhanced | edpf | 5 | [1.007553200437379, 1.2571008094186902] | -10.526593701684355 | 1.75384722093776 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [0.9697758040729755, 1.0423610626873998] | 0.08427118242758103 | 1.2482207918201724 | () |
| rtt-threshold | adaptive | 5 | [0.9590877932152949, 1.0324893071000856] | -0.14958919010701344 | 1.0749293743091657 | () |
| rtt-threshold | classic | 5 | [0.9317631771298628, 1.033975259568927] | 0.8222447564514734 | 1.0025425918102788 | () |
| rtt-threshold | edpf | 5 | [1.0060391958953654, 1.2435838488547066] | -10.610864884111937 | 1.0186110799356538 | () |
| rtt-threshold | enhanced | 5 | [0.9593604709502626, 1.0311661683041435] | -0.08427118242758103 | 1.110782910054326 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--I@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 9198664.533333335 | 9159710.933333334 | 55604.8025129737 | [9155499.733333332, 9259551.466666669] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.002108464339893451 | 0.0021568395852847866 | 0.0010987842247315966 | [0.0005099334332808736, 0.0035398454203603436] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 21.52959721990356 | 23.32726382887718 | 3.764270254330028 | [16.450105635881894, 24.903064457519218] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2826.4 | 2826.0 | 12.895735729302148 | [2807.0, 2843.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 47.10666666666667 | 47.1 | 0.21492892882170284 | [46.78333333333333, 47.38333333333333] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 2.4766666666666666 | 2.7 | 0.44010731014656473 | [1.8833333333333333, 2.85] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.002860624200900696 | 0.0029520156438026476 | 0.0002454004743425744 | [0.0024259976680332494, 0.003024667004828195] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.772269560957913 | 12.772849811320754 | 0.010065986915283829 | [12.757483653846156, 12.78306] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1250.6 | 1249.0 | 20.243517480912253 | [1220.0, 1270.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 61.401300000000006 | 61.441 | 0.06952661360946619 | [61.299, 61.455] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 49.8 | 50.0 | 11.777096416349828 | [37.0, 67.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 49.8 | 50.0 | 11.777096416349828 | [37.0, 67.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.0029752543434717364 | 0.0028463176942941696 | 0.0003951314372035349 | [0.0025073051182957864, 0.003523780809527399] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | 20337.2 | 19949.0 | 549.4694713994581 | [19925.0, 20948.0] | 0.0 |
| adaptive | episode[1].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 9158868.693333333 | 9156903.466666669 | 37978.178172957385 | [9102157.866666667, 9207788.8] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.01714135763209807 | 0.015763764970330135 | 0.015517148628373115 | [0.0029758274689050124, 0.04013551078769351] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 22.96410658348247 | 17.882228861940643 | 7.81646654934811 | [16.50776351429781, 33.10570281766446] | 0.0 |
| classic | switch_count | 5 | 0 | 17203.8 | 18899.0 | 7160.0198812573135 | [7513.0, 25387.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 286.7278915351411 | 314.98333333333335 | 119.33221506375223 | [125.21666666666668, 423.1096148397527] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 2.6266485003027724 | 2.0499658339027684 | 0.8846305864563827 | [1.9, 3.7666666666666666] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.032919307300709584 | 0.02340010310037558 | 0.02457421334241996 | [0.00840384175623142, 0.07122586974039169] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.777832373730044 | 12.767087169811322 | 0.02381829355382536 | [12.751660961538462, 12.809735660377362] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1291.0 | 1256.0 | 57.86622503671723 | [1241.0, 1359.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 62.09929999999999 | 61.8735 | 0.43765934241142274 | [61.677, 62.611] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 37.0 | 37.0 | 4.242640687119285 | [33.0, 43.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 37.0 | 37.0 | 4.242640687119285 | [33.0, 43.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.0027916196981391016 | 0.002847981893625047 | 0.00020969158155309382 | [0.0024612071451919934, 0.0029979821274142403] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | 20516.8 | 20897.0 | 570.175586990534 | [19841.0, 20952.0] | 0.0 |
| classic | episode[1].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 9051518.186666667 | 9155148.8 | 261947.74513079482 | [8584706.666666666, 9207437.866666667] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.00946634276310634 | 0.0 | 0.021167385916618965 | [0.0, 0.047331713815531695] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.012302426088979634 | 0.012890568502203742 | 0.007227256178061164 | [0.0015020913730924013, 0.02014558444736045] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 27.15624698969907 | 24.767875471084984 | 7.241238865573827 | [19.216788904854997, 37.21628879052332] | 0.0 |
| edpf | switch_count | 5 | 0 | 6242.8 | 6030.0 | 991.526701607173 | [4860.0, 7326.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 104.04666666666667 | 100.5 | 16.52544502678622 | [81.0, 122.1] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 3.08 | 2.8333333333333335 | 0.864050538645358 | [2.2, 4.283333333333333] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.07251685388269097 | 0.04221917115294287 | 0.08556932952034212 | [0.02240937655492693, 0.22461119028440424] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.718859117606112 | 12.776842264150943 | 0.13145385630721554 | [12.48385755102041, 12.784475471698116] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1340.6 | 1280.0 | 126.02102999102968 | [1274.0, 1565.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 64.25410000000001 | 64.326 | 0.43209437626518266 | [63.709, 64.871] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 188.8 | 39.0 | 342.34441721751506 | [24.0, 801.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 188.8 | 39.0 | 342.34441721751506 | [24.0, 801.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 483.2 | 0.0 | 1080.4680467278984 | [0.0, 2416.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.011156487796423451 | 0.0027674213014567397 | 0.018849478142577047 | [0.002414319935115152, 0.04486974588460923] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | 20524.6 | 20820.0 | 521.6510327795777 | [19950.0, 20956.0] | 0.0 |
| edpf | episode[1].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 8833939.52 | 9158131.733333332 | 753544.7981893552 | [7486636.266666667, 9211122.666666666] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.03625394321766562 | 0.0 | 0.08106628148711778 | [0.0, 0.1812697160883281] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.01774209640863683 | 0.005907791183566213 | 0.023887915546089857 | [0.00020304840810525526, 0.05778222343052988] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 22.803770123866776 | 17.94931403222365 | 11.347318543359199 | [16.003872587990898, 42.92092227907889] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2490.8 | 2796.0 | 801.105923083833 | [1070.0, 3001.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 41.513333333333335 | 46.6 | 13.35176538473055 | [17.833333333333332, 50.016666666666666] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 2.433333333333333 | 2.066666666666667 | 0.9017729450612524 | [1.8333333333333333, 4.016666666666667] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.07140427892682012 | 0.00293109063392019 | 0.15334302847178022 | [0.0025104442144800888, 0.34571242798602814] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.410040642096737 | 12.753521923076924 | 0.7757910744088609 | [11.022493023255814, 12.77317037735849] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1219.0 | 1232.0 | 35.552777669262355 | [1168.0, 1250.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 61.411699999999996 | 61.324 | 0.31887411309167163 | [61.175, 61.955] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 137.6 | 44.0 | 212.1610237531861 | [34.0, 517.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 137.6 | 44.0 | 212.1610237531861 | [34.0, 517.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 1838.8 | 0.0 | 4111.681797026613 | [0.0, 9194.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.020196450222227744 | 0.0029983028474448426 | 0.0387338504633173 | [0.002612822039922384, 0.08948463138433095] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | 19954.0 | None | [19936.0, +inf] | 0.2 |
| enhanced | episode[1].recovery_ms | 1 | 4 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 9159289.813333333 | 9155850.666666666 | 8402.820018488368 | [9154446.933333334, 9174274.666666666] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.0035517611861246136 | 0.001991136236270008 | 0.003409382137762178 | [0.0006104226837906068, 0.007968006556057339] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 23.20126691982378 | 24.756483577422557 | 5.600930037112297 | [16.746391138147647, 30.08412218164822] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 2929.2 | 2953.0 | 46.467192727772996 | [2855.0, 2969.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 48.81983816936385 | 49.21666666666667 | 0.7745238178409725 | [47.583333333333336, 49.483333333333334] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 2.6566566668333307 | 2.8333333333333335 | 0.6431841585068366 | [1.9166666666666667, 3.45] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.0024524251480391142 | 0.0024155978604704664 | 0.00029130033036647363 | [0.002108314665963895, 0.0029139720258685516] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.773004222786648 | 12.77405113207547 | 0.01660346402147795 | [12.757009230769231, 12.79828169811321] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1265.6 | 1259.0 | 43.310506808394656 | [1225.0, 1337.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 61.42100000000001 | 61.434 | 0.21295187249704975 | [61.167, 61.708] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 37.4 | 37.0 | 5.594640292279746 | [32.0, 46.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 37.4 | 37.0 | 5.594640292279746 | [32.0, 46.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.002564573598847846 | 0.0025361205041500154 | 0.000261635669193899 | [0.002244817113429288, 0.0029601975979033504] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | 20732.8 | 20924.0 | 437.7907034188826 | [19951.0, 20951.0] | 0.0 |
| rtt-threshold | episode[1].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=False | — | 1.0 | 0.0 |
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=False | — | 1.0 | 0.0 |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=False | — | 1.0 | 0.0 |
| edpf | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=False | — | 0.8 | 0.19999999999999996 |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=False | — | 1.0 | 0.0 |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9947023401173869, 1.0172918996028841] | 0.0 | None | () |
| adaptive | edpf | 5 | [0.994816480542745, 1.066898313745529] | 0.0 | None | () |
| adaptive | enhanced | 5 | [0.9994064600126369, 1.236810649916798] | 0.0 | None | () |
| adaptive | rtt-threshold | 5 | [0.9999808352019012, 1.0113261786124954] | 0.0 | None | () |
| classic | adaptive | 5 | [0.9830020276288112, 1.0053258745545808] | 0.0 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.9938309448999925, 1.0725804803270313] | 0.0 | None | () |
| classic | enhanced | 5 | [0.9940565768168398, 1.2157873766611198] | 0.0 | None | () |
| classic | rtt-threshold | 5 | [0.9921392368748209, 1.0058268803189452] | 0.0 | None | () |
| edpf | adaptive | 5 | [0.9372964481397754, 1.0052105283322479] | 0.0 | None | () |
| edpf | classic | 5 | [0.9323309703483496, 1.0062073485753942] | 0.0 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.936740125217791, 1.2233341927016195] | 0.0 | None | () |
| edpf | rtt-threshold | 5 | [0.9377635513302154, 1.0055957993177724] | 0.0 | None | () |
| enhanced | adaptive | 5 | [0.8085312008489509, 1.000593892486302] | 0.0 | None | () |
| enhanced | classic | 5 | [0.8225122411998304, 1.0059789586646992] | 0.0 | None | () |
| enhanced | edpf | 5 | [0.8174381178634379, 1.0675319366377107] | 0.0 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.8160466673041982, 1.0060367957071674] | 0.0 | None | () |
| rtt-threshold | adaptive | 5 | [0.9888006670330293, 1.0000191651653954] | 0.0 | None | () |
| rtt-threshold | classic | 5 | [0.9942068755240491, 1.007923044299649] | 0.0 | None | () |
| rtt-threshold | edpf | 5 | [0.994435339406182, 1.0663668880940216] | 0.0 | None | () |
| rtt-threshold | enhanced | 5 | [0.9939994285170016, 1.22542011390536] | 0.0 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--J@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 9311349.226666668 | 9336932.266666668 | 82351.78021895076 | [9195681.6, 9414839.466666669] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.27094299435047897 | 0.27086235279219956 | 0.006072584737998831 | [0.2629315160317283, 0.27993352839246655] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.025778318753885865 | 0.026840525672252452 | 0.008483355229253376 | [0.01629622364312175, 0.03399218810144544] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 59.12017025346104 | 66.8767796054874 | 19.451058511659333 | [33.98889605425171, 82.35749847334137] | 0.0 |
| adaptive | switch_count | 5 | 0 | 1072.2 | 1074.0 | 11.64903429473877 | [1057.0, 1085.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 17.869999999999997 | 17.9 | 0.1941505715789786 | [17.616666666666667, 18.083333333333332] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 6.866666666666667 | 7.75 | 2.2106748592530145 | [4.0, 9.466666666666669] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.45219885431463247 | 0.4527963235076577 | 0.003908870365369401 | [0.4460219217193198, 0.456497961618773] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.659345433263452 | 10.664105471698113 | 0.07967015733741815 | [10.541309999999998, 10.76502777777778] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1034.2 | 1042.0 | 80.2446259882866 | [906.0, 1120.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 65.4419 | 62.539 | 6.130401357170668 | [62.372, 76.3825] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 932.8 | 996.0 | 156.93852299547106 | [700.0, 1093.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 932.8 | 996.0 | 156.93852299547106 | [700.0, 1093.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 19319.6 | 19347.0 | 323.77584838897417 | [18828.0, 19709.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.13318556725578903 | 0.13603186331933606 | 0.005020153730130553 | [0.12679782662736602, 0.13768752286864253] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 9351004.693333333 | 9347460.266666668 | 93280.16603636845 | [9254813.866666667, 9499414.4] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.2678495527569102 | 0.27109843397349803 | 0.008113197485067447 | [0.2542464123305331, 0.27436560643131774] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.02675174140447526 | 0.020711196907959972 | 0.014351984602138143 | [0.01725008254562263, 0.05169618713345389] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 59.52622412988021 | 60.83733640137168 | 23.0454971536402 | [35.08988231246479, 94.8637301034009] | 0.0 |
| classic | switch_count | 5 | 0 | 8508.8 | 7496.0 | 4637.449913476155 | [4102.0, 14242.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 141.81264484480815 | 124.93333333333334 | 77.29011107951735 | [68.36666666666666, 237.36666666666667] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 6.953316611389811 | 7.066666666666666 | 2.689743454748717 | [4.166666666666667, 11.1] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.45317183161505764 | 0.4540188199340432 | 0.003061514024858265 | [0.4478654896357337, 0.4555442602439787] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.697050572327043 | 10.663082830188682 | 0.0836375363730101 | [10.63290169811321, 10.842999259259257] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1199.8 | 1170.0 | 105.26015390450463 | [1085.0, 1337.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 63.9671 | 64.087 | 0.5924968776289032 | [63.328, 64.636] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 791.0 | 886.0 | 302.0115891816074 | [450.0, 1152.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 791.0 | 886.0 | 302.0115891816074 | [450.0, 1152.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 19171.4 | 19473.0 | 700.8960693283991 | [17947.0, 19624.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.1357491030728022 | 0.13404001302413082 | 0.009032246747866795 | [0.12601436265709157, 0.15049233941203655] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [8978.0, +inf] | 0.8 |
| classic | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [5970.0, +inf] | 0.8 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [8954.0, +inf] | 0.8 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [5942.0, +inf] | 0.8 |
| classic | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 8754804.053333333 | 9352022.4 | 866458.4650748437 | [7722112.533333333, 9425542.933333334] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.31379824010431023 | 0.26968192784448575 | 0.0693166574123126 | [0.2537919337124108, 0.3951002101521841] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.028016269676134565 | 0.03521626835988467 | 0.02286137229616523 | [0.0024968874201519997, 0.051346427138201145] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 53.82223632723761 | 49.55466072461967 | 15.286596324084696 | [38.05262669784738, 74.98633526039208] | 0.0 |
| edpf | switch_count | 5 | 0 | 1773.0 | 2647.0 | 1352.9266425050546 | [77.0, 2886.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 29.55 | 44.11666666666667 | 22.548777375084242 | [1.2833333333333334, 48.1] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 5.826666666666666 | 5.016666666666667 | 1.4713750182888268 | [4.483333333333333, 7.45] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.5065542989889501 | 0.45724675224991845 | 0.0712951738215151 | [0.4498138061094571, 0.5854785478547855] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.144379636363634 | 10.672605 | 0.8339601634203873 | [9.15683, 10.876840181818183] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 951.8 | 916.0 | 326.44478859372225 | [510.0, 1280.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 269.30080000000004 | 75.758 | 273.28863616801743 | [66.435, 573.768] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 1013.6 | 928.0 | 315.23372281531044 | [743.0, 1507.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 1013.6 | 928.0 | 315.23372281531044 | [743.0, 1507.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 22466.4 | 19450.0 | 4797.844807827782 | [18255.0, 28013.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.16070648938257365 | 0.1347935673967837 | 0.0423754752787937 | [0.12372766342456458, 0.221010942348522] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | 7949.0 | None | [5973.0, +inf] | 0.4 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | 4950.0 | None | [2974.0, +inf] | 0.4 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | 7891.0 | None | [5945.0, +inf] | 0.4 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | 4899.0 | None | [2950.0, +inf] | 0.4 |
| edpf | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 9395222.293333333 | 9400626.666666666 | 103170.69961618108 | [9251655.466666669, 9532402.133333333] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.26429787827229023 | 0.2635086983860826 | 0.007906847937522592 | [0.2544734363747161, 0.2763006590940962] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.03799246271375409 | 0.045795577786432534 | 0.012767336443815648 | [0.019973839397404075, 0.04823656605153112] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 57.75971381847961 | 48.507404470907616 | 26.75811843004352 | [32.4917069193313, 86.86163135151551] | 0.0 |
| enhanced | switch_count | 5 | 0 | 1084.8 | 1077.0 | 24.406966218684367 | [1064.0, 1126.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 18.079939667672203 | 17.95 | 0.40677908476933855 | [17.733333333333334, 18.766666666666666] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 6.783314333649995 | 5.699905001583307 | 3.144337293427059 | [3.8, 10.35] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.44964390754591743 | 0.4496931974986054 | 0.004930467421377731 | [0.4416537308252263, 0.4542529679829451] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.735630685674355 | 10.699005740740745 | 0.13117323774750134 | [10.632661132075468, 10.958006] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 964.8 | 1050.0 | 325.4922426110951 | [396.0, 1180.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 63.2931 | 63.382 | 0.3244634802254346 | [62.852, 63.698] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 773.6 | 670.0 | 266.3630980447555 | [540.0, 1222.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 773.6 | 670.0 | 266.3630980447555 | [540.0, 1222.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 18950.0 | 18858.0 | 491.0005091647055 | [18374.0, 19703.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.13368356987272295 | 0.13003272158201737 | 0.018989074245662173 | [0.11668787069722436, 0.16458297357969262] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [9941.0, +inf] | 0.8 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [6958.0, +inf] | 0.8 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [9877.0, +inf] | 0.8 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [6937.0, +inf] | 0.8 |
| enhanced | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 9370656.96 | 9400275.733333332 | 74525.1573327434 | [9285871.466666669, 9459408.0] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.2646580691525515 | 0.262948596305185 | 0.005935825566715524 | [0.25852535855623227, 0.27178800479244486] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.02323107158599051 | 0.021731452079208458 | 0.01034154510465722 | [0.009837325660126545, 0.036327926646632286] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 50.47890709948504 | 43.26114235400868 | 18.4812115569383 | [35.12380438569871, 78.11149830546867] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 1124.4 | 1121.0 | 19.3597520645281 | [1105.0, 1155.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 18.73993727882313 | 18.683333333333337 | 0.32264393566583704 | [18.416666666666668, 19.25] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 5.906649722504625 | 5.083248612523125 | 2.1347603014942274 | [4.083333333333333, 9.066666666666666] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.4502485035375871 | 0.44886396936722217 | 0.003495194863741291 | [0.4461533945815844, 0.45490544206830513] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.715637220824599 | 10.733709629629631 | 0.09106749137047125 | [10.574863018867925, 10.814312407407408] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 923.8 | 1045.0 | 254.6874555214685 | [473.0, 1064.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 67.2717 | 63.392 | 6.266517569591582 | [62.13, 76.4765] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 860.8 | 755.0 | 325.2640465836948 | [547.0, 1396.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 860.8 | 755.0 | 325.2640465836948 | [547.0, 1396.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 18829.8 | 18845.0 | 367.76106373568155 | [18422.0, 19282.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.1303357474140161 | 0.13644955570865794 | 0.014164506852907908 | [0.11237168109206024, 0.1461981981981982] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [10973.0, +inf] | 0.8 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [7978.0, +inf] | 0.8 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [10950.0, +inf] | 0.8 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [7954.0, +inf] | 0.8 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[2] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[3] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[4] graded=False | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | post_restore_recovered_rate | — | 0.0 | — |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=False | — | 0.2 | 0.8 |
| classic | episode[2] graded=False | — | 0.2 | 0.8 |
| classic | episode[3] graded=False | — | 0.0 | 1.0 |
| classic | episode[4] graded=False | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | post_restore_recovered_rate | — | 0.2 | — |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=False | — | 0.6 | 0.4 |
| edpf | episode[2] graded=False | — | 0.6 | 0.4 |
| edpf | episode[3] graded=False | — | 0.0 | 1.0 |
| edpf | episode[4] graded=False | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| edpf | post_restore_recovered_rate | — | 0.6 | — |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=False | — | 0.2 | 0.8 |
| enhanced | episode[2] graded=False | — | 0.2 | 0.8 |
| enhanced | episode[3] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[4] graded=False | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | post_restore_recovered_rate | — | 0.2 | — |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 0.8 | — |
| rtt-threshold | episode[1] graded=False | — | 0.2 | 0.8 |
| rtt-threshold | episode[2] graded=False | — | 0.2 | 0.8 |
| rtt-threshold | episode[3] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[4] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | post_restore_recovered_rate | — | 0.2 | — |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9830618050168091, 1.0131611246435925] | -0.023608118129847533 | None | () |
| adaptive | edpf | 5 | [0.9756129344527803, 1.2005498875230067] | 0.11804249477138096 | None | () |
| adaptive | enhanced | 5 | [0.9725545779184922, 1.0092174638698175] | 0.7353654406116961 | None | () |
| adaptive | rtt-threshold | 5 | [0.9782353050977172, 1.0054987623060787] | 0.7913756487014534 | None | () |
| classic | adaptive | 5 | [0.9870098404651854, 1.0172300407733788] | 0.023608118129847533 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.981886553605004, 1.212206594105751] | 0.1416506129012285 | None | () |
| classic | enhanced | 5 | [0.981997570224202, 1.010355422372264] | 0.7589735587415436 | None | () |
| classic | rtt-threshold | 5 | [0.9823594880356148, 1.0213946117274169] | 0.8149837668313009 | None | () |
| edpf | adaptive | 5 | [0.8329516419040408, 1.0249966607514265] | -0.11804249477138096 | None | () |
| edpf | classic | 5 | [0.8249418909799806, 1.0184475959350827] | -0.1416506129012285 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.8100909325184995, 1.013826195804726] | 0.6173229458403151 | None | () |
| edpf | rtt-threshold | 5 | [0.820864342603473, 1.0100905122730104] | 0.6733331539300724 | None | () |
| enhanced | adaptive | 5 | [0.9908667217920771, 1.0282199299706634] | -0.7353654406116961 | None | () |
| enhanced | classic | 5 | [0.9897507133203185, 1.0183324585738922] | -0.7589735587415436 | None | () |
| enhanced | edpf | 5 | [0.9863623608642786, 1.2344293212751936] | -0.6173229458403151 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.9937859395288443, 1.014489472492642] | 0.05601020808975732 | None | () |
| rtt-threshold | adaptive | 5 | [0.9945313087273548, 1.0222489362108116] | -0.7913756487014534 | None | () |
| rtt-threshold | classic | 5 | [0.9790535298681147, 1.0179572877131366] | -0.8149837668313009 | None | () |
| rtt-threshold | edpf | 5 | [0.9900102890281548, 1.2182280897089233] | -0.6733331539300724 | None | () |
| rtt-threshold | enhanced | 5 | [0.9857174737781744, 1.0062529164722354] | -0.05601020808975732 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--K@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 12001674.346666668 | 12002446.4 | 2514.1379323955143 | [11997357.866666667, 12003850.133333333] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.1241987972447067 | 0.11709356705187032 | 0.014151140265594738 | [0.11202375434897185, 0.14188784123063436] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 30.95011759196076 | 34.43339668041618 | 9.358641050431906 | [17.78169291140814, 41.213820098098225] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2327.4 | 2365.0 | 88.74852111443886 | [2208.0, 2411.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 38.70161978531501 | 39.40090630414501 | 1.5203174101818453 | [36.78651161240878, 40.11914270500532] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 4.63105958492554 | 5.158413205537806 | 1.396444816333944 | [2.6656892472759988, 6.176230668065059] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.4552653728323385 | 0.4550199742216458 | 0.0005469392234983552 | [0.4547565291188292, 0.4559505162548349] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.418337647058824 | 12.418113235294118 | 0.0032363968040647812 | [12.413698529411764, 12.422639705882355] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1990.2 | 1991.0 | 2.7748873851023212 | [1987.0, 1994.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 44.8707 | 44.8005 | 0.5105931109993539 | [44.477500000000006, 45.729] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 94.8 | 96.0 | 14.601369798755185 | [75.0, 113.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 94.8 | 96.0 | 14.601369798755185 | [75.0, 113.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.000807841122935116 | 0.0007494819756932708 | 9.696036040372594e-05 | [0.0007195089718363631, 0.0009400981227415612] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[9].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[9].recovery_ms | 5 | 0 | 1420.0 | 1424.0 | 28.151376520518493 | [1376.0, 1452.0] | 0.0 |
| adaptive | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[18].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[18].recovery_ms | 5 | 0 | 1423.8 | 1450.0 | 67.00522367696416 | [1311.0, 1475.0] | 0.0 |
| adaptive | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[28].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[28].recovery_ms | 5 | 0 | 1422.2 | 1451.0 | 59.50378139244597 | [1354.0, 1479.0] | 0.0 |
| adaptive | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 12002341.120000001 | 12002621.866666667 | 1956.2762849630371 | [11999463.466666669, 12004902.933333334] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.15382110988283174 | 0.1467894137694502 | 0.015893454923801547 | [0.13791419711053823, 0.17881250297155085] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 38.057815353597704 | 40.882347713254774 | 12.834045171133129 | [16.000715409764542, 48.65532456989471] | 0.0 |
| classic | switch_count | 5 | 0 | 14339.2 | 14332.0 | 466.2335251781021 | [13647.0, 14889.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 238.7464656692335 | 238.58831363409357 | 7.858588793878598 | [227.18116895007577, 248.0383827277725] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 5.704343639589032 | 6.1212947869190595 | 1.9243640031798777 | [2.3972032628599966, 7.291371876612675] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.4492995921782671 | 0.4495689585456192 | 0.0008432390476878656 | [0.44831274053068665, 0.4502063129335196] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.418479411764705 | 12.418325000000005 | 0.0029712530196733286 | [12.415163235294116, 12.422352941176468] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1995.2 | 1994.0 | 2.6832815729997477 | [1993.0, 1999.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 46.211 | 46.3115 | 0.43941608982831 | [45.6425, 46.6465] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 94.2 | 87.0 | 13.516656391282572 | [83.0, 113.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 94.2 | 87.0 | 13.516656391282572 | [83.0, 113.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.0008284171752154873 | 0.0008668439533961183 | 9.671704469502279e-05 | [0.0006755665212730024, 0.0009105996739465684] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[9].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[9].recovery_ms | 5 | 0 | 1424.2 | 1401.0 | 39.996249824202266 | [1388.0, 1477.0] | 0.0 |
| classic | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[18].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[18].recovery_ms | 5 | 0 | 1400.4 | 1398.0 | 60.401158929278836 | [1310.0, 1475.0] | 0.0 |
| classic | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[28].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[28].recovery_ms | 5 | 0 | 1441.0 | 1472.0 | 48.55409354524086 | [1375.0, 1479.0] | 0.0 |
| classic | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 10500978.133333333 | 9873158.4 | 1396601.9369080404 | [9119880.0, 12004727.466666669] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.12336415687147573 | 0.1755466630697061 | 0.11465834523165087 | [0.0, 0.23647474598733617] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.21629313326473626 | 0.21713875229970758 | 0.047768226459215093 | [0.1583145257242845, 0.28806522913458477] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 62.069417231747124 | 61.99425243150715 | 27.353450933763828 | [26.98936738877626, 94.5918879049578] | 0.0 |
| edpf | switch_count | 5 | 0 | 1801.0 | 828.0 | 1720.672978808001 | [468.0, 4347.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 29.98573546047196 | 13.735443415945058 | 28.668400461827172 | [7.796491578789545, 72.40052630702354] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 7.791937361171205 | 7.363353157745681 | 2.7164631597474362 | [4.047976011994003, 10.767003378209717] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.43819401012734394 | 0.4380400728104425 | 0.010049358220483981 | [0.4224438753351006, 0.4486588688144644] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 15.461154248252294 | 16.330760535714287 | 1.8966992717416424 | [12.887244117647056, 17.382865185185185] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1661.6 | 1607.0 | 194.91485320518805 | [1504.0, 1975.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 418.0209 | 607.4865 | 343.8418199074976 | [43.3365, 711.0135] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 17435.8 | 24572.0 | 11829.813658718382 | [2349.0, 28506.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 17435.8 | 24572.0 | 11829.813658718382 | [2349.0, 28506.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 8282.0 | 11713.0 | 7715.621718306309 | [0.0, 16059.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.2966587609145865 | 0.4085397425554204 | 0.19219054331567187 | [0.037916100543478264, 0.44743537741067146] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| edpf | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [1364.0, +inf] | 0.6 |
| edpf | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [0.0, +inf] | 0.6 |
| edpf | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [1418.0, +inf] | 0.6 |
| edpf | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[28].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| edpf | episode[28].recovery_ms | 5 | 0 | +inf | 1463.0 | None | [1390.0, +inf] | 0.2 |
| edpf | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 12001498.88 | 12001042.666666666 | 1846.9898383879442 | [11999638.933333334, 12004376.533333331] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.12821019506661185 | 0.13765563188892996 | 0.016567373575072222 | [0.10854653733831773, 0.14152015116203032] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 34.59685269148706 | 32.54987162463941 | 12.987433250248937 | [16.216307954530006, 51.43997488218246] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2313.6 | 2256.0 | 90.79262084552907 | [2234.0, 2419.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 38.521480195278485 | 37.58496601359456 | 1.523794267255807 | [37.169525647638224, 40.27571968498693] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 5.184077972974031 | 4.881543434074173 | 1.9437140256092602 | [2.432360389177662, 7.703442423839077] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.4550076463954512 | 0.4550007600547239 | 0.0009224638446410757 | [0.4539000264734904, 0.4560857891960412] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.418723064791132 | 12.418251470588242 | 0.0037283283084357756 | [12.41408235294118, 12.423947058823526] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1990.8 | 1991.0 | 2.3874672772626644 | [1988.0, 1994.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 45.349900000000005 | 45.4285 | 0.18385673770629296 | [45.129999999999995, 45.515] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 96.2 | 99.0 | 14.20211251891774 | [79.0, 110.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 96.2 | 99.0 | 14.20211251891774 | [79.0, 110.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.0007819032291400843 | 0.0007961552937089257 | 0.0001300992369418314 | [0.0006022945955077637, 0.000910265445149166] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[9].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[9].recovery_ms | 5 | 0 | 1440.4 | 1464.0 | 38.99102460823516 | [1384.0, 1471.0] | 0.0 |
| enhanced | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[18].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[18].recovery_ms | 5 | 0 | 1418.2 | 1420.0 | 71.26499842138496 | [1305.0, 1479.0] | 0.0 |
| enhanced | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[28].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[28].recovery_ms | 5 | 0 | 1440.2 | 1467.0 | 43.80296793597439 | [1388.0, 1477.0] | 0.0 |
| enhanced | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 11999603.84 | 11999638.933333334 | 1576.2728427101101 | [11997006.933333334, 12000867.2] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.3365777318305527 | 0.3355761036758704 | 0.005653389882792706 | [0.3303953031645115, 0.34429516241515357] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 39.64593102179334 | 40.44566140945663 | 2.7547247808784503 | [35.77519242387194, 42.67731133649868] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 972.2 | 981.0 | 32.6527181104422 | [930.0, 1010.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 16.165272021887148 | 16.238764463425536 | 0.541063800704634 | [15.469576499550882, 16.807841440481937] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 5.932553182366304 | 6.057479489441015 | 0.40891635644822677 | [5.356132938554177, 6.395416618090368] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.4458190606576837 | 0.44597981770833334 | 0.0006632513737372212 | [0.44489449963732974, 0.4466153558569907] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 12.41620588235294 | 12.418508823529413 | 0.005617721838891219 | [12.406735294117649, 12.420748529411764] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1997.4 | 1995.0 | 9.343446901438464 | [1985.0, 2009.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 42.4849 | 42.5445 | 0.12373429193235007 | [42.3125, 42.597] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 61.6 | 67.0 | 29.02240513809977 | [12.0, 88.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 61.6 | 67.0 | 29.02240513809977 | [12.0, 88.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.0005463741209942603 | 0.0005432229269438572 | 0.0001516051051093044 | [0.0003821955665314282, 0.0007195301027900147] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[9].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | episode[9].recovery_ms | 5 | 0 | 1454.6 | 1477.0 | 51.22792207380659 | [1363.0, 1480.0] | 0.0 |
| rtt-threshold | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[18].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | episode[18].recovery_ms | 5 | 0 | 1425.4 | 1398.0 | 39.88483421051164 | [1394.0, 1472.0] | 0.0 |
| rtt-threshold | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[28].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | episode[28].recovery_ms | 5 | 0 | 1393.4 | 1387.0 | 15.076471735787521 | [1380.0, 1418.0] | 0.0 |
| rtt-threshold | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[2] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[3] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[4] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[5] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[6] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[7] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[8] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[9] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[10] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[11] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[12] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[13] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[14] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[15] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[16] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[17] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[18] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[19] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[20] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[21] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[22] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[23] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[24] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[25] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[26] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[27] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[28] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[29] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[30] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[31] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[32] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[33] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[34] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[35] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[36] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[37] graded=False | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=False | — | 0.0 | 1.0 |
| classic | episode[2] graded=False | — | 0.0 | 1.0 |
| classic | episode[3] graded=False | — | 0.0 | 1.0 |
| classic | episode[4] graded=False | — | 0.0 | 1.0 |
| classic | episode[5] graded=False | — | 0.0 | 1.0 |
| classic | episode[6] graded=False | — | 0.0 | 1.0 |
| classic | episode[7] graded=False | — | 0.0 | 1.0 |
| classic | episode[8] graded=False | — | 0.0 | 1.0 |
| classic | episode[9] graded=True | — | 1.0 | 0.0 |
| classic | episode[10] graded=True | — | 0.0 | 1.0 |
| classic | episode[11] graded=False | — | 0.0 | 1.0 |
| classic | episode[12] graded=False | — | 0.0 | 1.0 |
| classic | episode[13] graded=False | — | 0.0 | 1.0 |
| classic | episode[14] graded=False | — | 0.0 | 1.0 |
| classic | episode[15] graded=False | — | 0.0 | 1.0 |
| classic | episode[16] graded=False | — | 0.0 | 1.0 |
| classic | episode[17] graded=False | — | 0.0 | 1.0 |
| classic | episode[18] graded=True | — | 1.0 | 0.0 |
| classic | episode[19] graded=False | — | 0.0 | 1.0 |
| classic | episode[20] graded=True | — | 0.0 | 1.0 |
| classic | episode[21] graded=False | — | 0.0 | 1.0 |
| classic | episode[22] graded=False | — | 0.0 | 1.0 |
| classic | episode[23] graded=False | — | 0.0 | 1.0 |
| classic | episode[24] graded=False | — | 0.0 | 1.0 |
| classic | episode[25] graded=False | — | 0.0 | 1.0 |
| classic | episode[26] graded=False | — | 0.0 | 1.0 |
| classic | episode[27] graded=False | — | 0.0 | 1.0 |
| classic | episode[28] graded=True | — | 1.0 | 0.0 |
| classic | episode[29] graded=True | — | 0.0 | 1.0 |
| classic | episode[30] graded=False | — | 0.0 | 1.0 |
| classic | episode[31] graded=False | — | 0.0 | 1.0 |
| classic | episode[32] graded=False | — | 0.0 | 1.0 |
| classic | episode[33] graded=False | — | 0.0 | 1.0 |
| classic | episode[34] graded=False | — | 0.0 | 1.0 |
| classic | episode[35] graded=False | — | 0.0 | 1.0 |
| classic | episode[36] graded=False | — | 0.0 | 1.0 |
| classic | episode[37] graded=False | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=False | — | 0.0 | 1.0 |
| edpf | episode[2] graded=False | — | 0.0 | 1.0 |
| edpf | episode[3] graded=False | — | 0.0 | 1.0 |
| edpf | episode[4] graded=False | — | 0.0 | 1.0 |
| edpf | episode[5] graded=False | — | 0.0 | 1.0 |
| edpf | episode[6] graded=False | — | 0.0 | 1.0 |
| edpf | episode[7] graded=False | — | 0.0 | 1.0 |
| edpf | episode[8] graded=False | — | 0.0 | 1.0 |
| edpf | episode[9] graded=True | — | 0.4 | 0.6 |
| edpf | episode[10] graded=True | — | 0.0 | 1.0 |
| edpf | episode[11] graded=False | — | 0.0 | 1.0 |
| edpf | episode[12] graded=False | — | 0.0 | 1.0 |
| edpf | episode[13] graded=False | — | 0.0 | 1.0 |
| edpf | episode[14] graded=False | — | 0.0 | 1.0 |
| edpf | episode[15] graded=False | — | 0.0 | 1.0 |
| edpf | episode[16] graded=False | — | 0.0 | 1.0 |
| edpf | episode[17] graded=False | — | 0.0 | 1.0 |
| edpf | episode[18] graded=True | — | 0.4 | 0.6 |
| edpf | episode[19] graded=False | — | 0.0 | 1.0 |
| edpf | episode[20] graded=True | — | 0.0 | 1.0 |
| edpf | episode[21] graded=False | — | 0.0 | 1.0 |
| edpf | episode[22] graded=False | — | 0.0 | 1.0 |
| edpf | episode[23] graded=False | — | 0.0 | 1.0 |
| edpf | episode[24] graded=False | — | 0.0 | 1.0 |
| edpf | episode[25] graded=False | — | 0.0 | 1.0 |
| edpf | episode[26] graded=False | — | 0.0 | 1.0 |
| edpf | episode[27] graded=False | — | 0.0 | 1.0 |
| edpf | episode[28] graded=True | — | 0.8 | 0.19999999999999996 |
| edpf | episode[29] graded=True | — | 0.0 | 1.0 |
| edpf | episode[30] graded=False | — | 0.0 | 1.0 |
| edpf | episode[31] graded=False | — | 0.0 | 1.0 |
| edpf | episode[32] graded=False | — | 0.0 | 1.0 |
| edpf | episode[33] graded=False | — | 0.0 | 1.0 |
| edpf | episode[34] graded=False | — | 0.0 | 1.0 |
| edpf | episode[35] graded=False | — | 0.0 | 1.0 |
| edpf | episode[36] graded=False | — | 0.0 | 1.0 |
| edpf | episode[37] graded=False | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 0.4 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[2] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[4] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[6] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[7] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[8] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[9] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[10] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[11] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[12] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[13] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[14] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[15] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[16] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[17] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[18] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[19] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[20] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[21] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[22] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[23] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[24] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[25] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[26] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[27] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[28] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[29] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[30] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[31] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[32] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[33] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[34] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[35] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[36] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[37] graded=False | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[2] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[3] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[4] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[5] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[6] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[7] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[8] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[9] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[10] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[11] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[12] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[13] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[14] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[15] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[16] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[17] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[18] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[19] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[20] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[21] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[22] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[23] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[24] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[25] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[26] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[27] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[28] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[29] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[30] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[31] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[32] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[33] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[34] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[35] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[36] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[37] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [0.9998245254876728, 1.000102333196889] | 0.0 | 1.0 | () |
| adaptive | edpf | 5 | [0.9997661365762394, 1.3160750360750362] | -17.55466630697061 | 0.4197280879375181 | () |
| adaptive | enhanced | 5 | [0.9994153243488177, 1.000292436139258] | 0.0 | 0.9972954699121027 | () |
| adaptive | rtt-threshold | 5 | [1.0000292517404785, 1.0002632079196339] | 0.0 | 0.9833935018050541 | () |
| classic | adaptive | 5 | [0.9998976772741227, 1.0001755053090358] | 0.0 | 1.0 | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| classic | edpf | 5 | [0.9997661365762394, 1.3161135161135162] | -17.55466630697061 | 0.40584321666184553 | () |
| classic | enhanced | 5 | [0.9995907270441726, 1.0004386798660565] | 0.0 | 1.0 | () |
| classic | rtt-threshold | 5 | [1.000087726993596, 1.0004386798660565] | 0.0 | 0.9927797833935018 | () |
| edpf | adaptive | 5 | [0.7598350949519758, 1.000233918128655] | 17.55466630697061 | 2.38249483115093 | () |
| edpf | classic | 5 | [0.7598128791754988, 1.000233918128655] | 17.55466630697061 | 2.4640057020669994 | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| edpf | enhanced | 5 | [0.7599239710505155, 1.000219295039546] | 17.55466630697061 | 2.474588403722262 | () |
| edpf | rtt-threshold | 5 | [0.7600128679427376, 1.0003216656431853] | 17.55466630697061 | 2.474588403722262 | () |
| enhanced | adaptive | 5 | [0.9997076493546361, 1.0005850176967852] | 0.0 | 1.0027118644067796 | () |
| enhanced | classic | 5 | [0.9995615124895859, 1.000409440528763] | 0.0 | 1.0 | () |
| enhanced | edpf | 5 | [0.9997807530402245, 1.315921115921116] | -17.55466630697061 | 0.40410760775238647 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [0.9999561365032021, 1.0006142865500496] | 0.0 | 1.0 | () |
| rtt-threshold | adaptive | 5 | [0.9997368613405453, 0.9999707491151607] | 0.0 | 1.0168869309838473 | () |
| rtt-threshold | classic | 5 | [0.9995615124895859, 0.9999122807017543] | 0.0 | 1.0072727272727273 | () |
| rtt-threshold | edpf | 5 | [0.999678437792329, 1.3157671957671957] | -17.55466630697061 | 0.40410760775238647 | () |
| rtt-threshold | enhanced | 5 | [0.9993860905662586, 1.0000438654208885] | 0.0 | 1.0 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--L@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 5876624.32 | 6441732.266666667 | 1165448.0717324114 | [3839912.533333333, 6586141.333333333] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.41225883170453886 | 0.39716755004895177 | 0.033664087926054166 | [0.39024640657084186, 0.4704113924050633] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.04815346338199846 | 0.04616075496584207 | 0.0400505462089524 | [0.0035179199397738226, 0.11051131021850358] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 113.13568094956574 | 94.0993655966608 | 32.91779474858006 | [86.03925090381703, 161.20421830731175] | 0.0 |
| adaptive | switch_count | 5 | 0 | 176.4 | 166.0 | 20.525593779474445 | [163.0, 212.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 2.9399902223851826 | 2.7666666666666666 | 0.3420934685375233 | [2.716666666666667, 3.533333333333333] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 8.413289334066654 | 7.3 | 3.2983078383063127 | [4.516666666666667, 13.199780003666604] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.5794735056297907 | 0.5726358533888937 | 0.01557462227085594 | [0.5688083200615668, 0.6067384952520087] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.431589505146013 | 10.136111764705884 | 0.7420105067165239 | [10.049916842105263, 11.75589] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1248.8 | 1266.0 | 192.71403685253443 | [928.0, 1435.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 534.0880999999999 | 530.2325 | 18.585418896274575 | [517.528, 564.85] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 1553.8 | 1755.0 | 510.2447451958717 | [681.0, 1960.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 1553.8 | 1755.0 | 510.2447451958717 | [681.0, 1960.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 22148.4 | 22906.0 | 2438.5453245736485 | [17838.0, 23663.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.21912997686132352 | 0.2186652737357909 | 0.024511153153302934 | [0.18853029951242165, 0.25500111549044396] | 0.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| adaptive | load[1].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[2].reached_ms | 5 | 0 | +inf | +inf | None | [10000.0, +inf] | 0.8 |
| classic | useful_goodput_bps | 5 | 0 | 5056528.213333334 | 4888325.866666666 | 875170.8279388942 | [4316830.933333334, 6537361.6] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.4311906197134377 | 0.4435933727416535 | 0.026859684341429973 | [0.3892875490959712, 0.4556933018630124] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.03025772142729251 | 0.027900710414052315 | 0.016350282593593907 | [0.013498112782117322, 0.05417307000760718] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 103.01417050787731 | 98.73864382854563 | 38.50140491620006 | [49.07575758379619, 148.28336892724957] | 0.0 |
| classic | switch_count | 5 | 0 | 870.8 | 890.0 | 39.59419149319759 | [809.0, 908.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 14.513282889729618 | 14.833333333333334 | 0.6598439568255859 | [13.483333333333333, 15.133081115314743] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 6.583323055726849 | 6.033333333333333 | 2.979551781472135 | [3.0832819453009117, 10.65] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.5838143806502927 | 0.5866270959271315 | 0.008274247487173578 | [0.5697742352793891, 0.5908277917908736] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.339648421827885 | 10.451788103448274 | 0.23746970880994764 | [9.969994210526314, 10.54032996] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1255.8 | 1387.0 | 244.94019678280654 | [896.0, 1469.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 531.1075000000001 | 528.7835 | 22.372980361364483 | [504.443, 566.45] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 1703.4 | 1746.0 | 242.52175160178933 | [1446.0, 1998.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 1703.4 | 1746.0 | 242.52175160178933 | [1446.0, 1998.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 20931.0 | 20082.0 | 1549.2404590637311 | [19885.0, 23490.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.21926559805148843 | 0.2084871451439025 | 0.020308942900078366 | [0.2027129859387924, 0.2491533972188198] | 0.0 |
| classic | load[0].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| classic | load[1].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[2].reached_ms | 5 | 0 | +inf | +inf | None | [10000.0, +inf] | 0.8 |
| edpf | useful_goodput_bps | 5 | 0 | 4926928.533333333 | 4704261.333333333 | 1232078.7802609953 | [3553200.0, 6451909.333333333] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.45821095734860773 | 0.46484994994261714 | 0.051076393065174835 | [0.3977811396873424, 0.5204589793613308] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.09136099305494383 | 0.07945156654606203 | 0.058357476653382485 | [0.031349590782101394, 0.1774022719147023] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 100.14915858117547 | 109.81032352331044 | 29.836329200802922 | [65.80184869961589, 137.34098840481818] | 0.0 |
| edpf | switch_count | 5 | 0 | 399.0 | 465.0 | 217.7303378034398 | [18.0, 572.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 6.6499467786648 | 7.75 | 3.6288198339037483 | [0.29999500008333196, 9.533333333333331] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 6.033270834374983 | 6.099898335027749 | 1.7530424285239334 | [3.3, 8.116666666666667] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.5959184012484353 | 0.5929779146733075 | 0.025213508344930444 | [0.5705091684075035, 0.6228273957940572] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.186297111783322 | 10.16762783783784 | 0.9953534694142004 | [8.618385925925928, 11.150941739130433] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1078.2 | 1046.0 | 229.38548341165796 | [871.0, 1451.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 535.7674 | 537.935 | 6.289715955430749 | [528.755, 542.537] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 1827.8 | 1989.0 | 387.73663742287755 | [1392.0, 2251.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 1827.8 | 1989.0 | 387.73663742287755 | [1392.0, 2251.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 22575.0 | 23476.0 | 2941.6096783903877 | [19037.0, 26423.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.23933543222682302 | 0.2405104053455702 | 0.012587969102058744 | [0.21860567947332216, 0.25148505441239366] | 0.0 |
| edpf | load[0].reached_ms | 5 | 0 | +inf | 3000.0 | None | [3000.0, +inf] | 0.2 |
| edpf | load[1].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[2].reached_ms | 5 | 0 | +inf | +inf | None | [10000.0, +inf] | 0.8 |
| enhanced | useful_goodput_bps | 5 | 0 | 5676697.6 | 5781977.6 | 844336.3835739718 | [4804101.866666666, 6554381.866666666] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.41339832571865764 | 0.40722634299842664 | 0.025371090380093796 | [0.38981618423832665, 0.44287793539417375] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.026360444054734377 | 0.031877995869110554 | 0.01695486231040076 | [0.006492293325519705, 0.047190438598778583] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 104.72866387374499 | 94.59320689767969 | 40.70810050372439 | [52.45517413952144, 159.79657701884835] | 0.0 |
| enhanced | switch_count | 5 | 0 | 181.0 | 176.0 | 20.334699407662754 | [165.0, 216.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 3.01665672238796 | 2.933333333333333 | 0.3389128800967097 | [2.75, 3.6] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 7.443311833691661 | 7.75 | 2.833480006748512 | [3.15, 10.25] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.5807134397479015 | 0.5809228197934698 | 0.01082919864409863 | [0.567170287130027, 0.5942343415436536] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.292981728042328 | 10.317618214285714 | 0.22136711022205013 | [9.94527, 10.509505277777777] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1003.0 | 1140.0 | 579.1528295709173 | [0.0, 1410.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 544.1553 | 543.208 | 18.32229973147472 | [515.3865000000001, 561.367] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 1625.2 | 1583.0 | 180.1837950538283 | [1383.0, 1861.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 1625.2 | 1583.0 | 180.1837950538283 | [1383.0, 1861.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 21684.8 | 21741.0 | 1255.4989844679287 | [20387.0, 23510.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.2153125817456679 | 0.20598792160437557 | 0.027398333947152725 | [0.180349932705249, 0.2478070941813734] | 0.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| enhanced | load[1].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[2].reached_ms | 5 | 0 | +inf | +inf | None | [10000.0, +inf] | 0.8 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 5476490.133333334 | 5307691.2 | 575315.1848238088 | [4705840.533333333, 6193096.0] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.4154837532017773 | 0.4084675663909447 | 0.01982926372597209 | [0.39051139864448553, 0.43867221735319895] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.04927810536080635 | 0.03179755829341516 | 0.038154588017696944 | [0.010892398832173439, 0.10060262852052268] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 106.73756894715461 | 132.3442428574516 | 45.28598956923026 | [48.483101905652184, 150.87929435162238] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 212.6 | 200.0 | 26.903531366718383 | [192.0, 259.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 3.5433115559185127 | 3.3333333333333335 | 0.4484089038862638 | [3.199946667555541, 4.316666666666666] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 7.4599342788731295 | 9.71650472492125 | 3.526935254205798 | [3.216666666666667, 10.35] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.5795239613297005 | 0.5764510822736886 | 0.006874684570117433 | [0.573663489432242, 0.5906074305917811] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.375017074502775 | 10.273163 | 0.18841823959097762 | [10.19147882352941, 10.633067419354838] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1248.6 | 1362.0 | 181.4739650748834 | [960.0, 1372.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 550.1073 | 556.38 | 12.772624405344441 | [533.882, 563.809] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 1675.2 | 1632.0 | 238.651838459292 | [1377.0, 1997.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 1675.2 | 1632.0 | 238.651838459292 | [1377.0, 1997.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 21403.2 | 21939.0 | 1134.3664310971126 | [20021.0, 22518.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.21387213077571068 | 0.22736930461925872 | 0.031157049292300856 | [0.17122337638319515, 0.2472104831108814] | 0.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| rtt-threshold | load[1].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[2].reached_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | load[1] graded=False | 0.0 | 0.0 | 1.0 |
| adaptive | load[2] graded=True | 0.0 | 0.2 | 0.8 |
| adaptive | overload_no_collapse_rate | — | 0.0 | — |
| adaptive | burst_recovered_rate | — | 0.2 | — |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | load[1] graded=False | 0.0 | 0.0 | 1.0 |
| classic | load[2] graded=True | 0.0 | 0.2 | 0.8 |
| classic | overload_no_collapse_rate | — | 0.0 | — |
| classic | burst_recovered_rate | — | 0.2 | — |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | load[0] graded=True | 0.0 | 0.8 | 0.19999999999999996 |
| edpf | load[1] graded=False | 0.0 | 0.0 | 1.0 |
| edpf | load[2] graded=True | 0.0 | 0.2 | 0.8 |
| edpf | overload_no_collapse_rate | — | 0.0 | — |
| edpf | burst_recovered_rate | — | 0.2 | — |
| edpf | settled_rate | — | 0.8 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | load[1] graded=False | 0.0 | 0.0 | 1.0 |
| enhanced | load[2] graded=True | 0.0 | 0.2 | 0.8 |
| enhanced | overload_no_collapse_rate | — | 0.0 | — |
| enhanced | burst_recovered_rate | — | 0.2 | — |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 0.8 | — |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | load[1] graded=False | 0.0 | 0.0 | 1.0 |
| rtt-threshold | load[2] graded=True | 0.0 | 0.0 | 1.0 |
| rtt-threshold | overload_no_collapse_rate | — | 0.0 | — |
| rtt-threshold | burst_recovered_rate | — | 0.0 | — |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.7855271187049069, 1.5256889683765547] | -4.642582269270173 | None | () |
| adaptive | edpf | 5 | [0.595159097090019, 1.6786666666666668] | -6.768239989366537 | None | () |
| adaptive | enhanced | 5 | [0.664117504248604, 1.360631241344121] | -1.005879294947487 | None | () |
| adaptive | rtt-threshold | 5 | [0.6200311658875195, 1.399567470822924] | -1.1300016341992958 | None | () |
| classic | adaptive | 5 | [0.6554415878513388, 1.273030524584171] | 4.642582269270173 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.7300296735905044, 1.8398518518518518] | -2.1256577200963633 | None | () |
| classic | enhanced | 5 | [0.6586175509985545, 1.0462398188392565] | 3.636702974322686 | None | () |
| classic | rtt-threshold | 5 | [0.7685069008782935, 1.2329406314117413] | 3.5125806350708775 | None | () |
| edpf | adaptive | 5 | [0.5957108816521048, 1.6802229939681959] | 6.768239989366537 | None | () |
| edpf | classic | 5 | [0.5435220227071423, 1.3698073327371758] | 2.1256577200963633 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.5527199279416983, 1.1158655013352756] | 5.7623606944190495 | None | () |
| edpf | rtt-threshold | 5 | [0.6701303858627308, 1.2565718334016929] | 5.638238355167241 | None | () |
| enhanced | adaptive | 5 | [0.7349529906517024, 1.5057576311460428] | 1.005879294947487 | None | () |
| enhanced | classic | 5 | [0.9558038052016057, 1.5183318429395982] | -3.636702974322686 | None | () |
| enhanced | edpf | 5 | [0.8961653521892847, 1.8092345679012345] | -5.7623606944190495 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.8196809464061661, 1.3928185241806181] | -0.12412233925180871 | None | () |
| rtt-threshold | adaptive | 5 | [0.7145064606367391, 1.6128221531712668] | 1.1300016341992958 | None | () |
| rtt-threshold | classic | 5 | [0.811069060847626, 1.3012244897959184] | -3.5125806350708775 | None | () |
| rtt-threshold | edpf | 5 | [0.7958160237388724, 1.492246913580247] | -5.638238355167241 | None | () |
| rtt-threshold | enhanced | 5 | [0.7179686245114312, 1.2199868795101683] | 0.12412233925180871 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 8951888.213333333 | 8965176.888888888 | 123185.33717601628 | [8825856.355555555, 9112334.933333334] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.06685761862072354 | 0.0667220038573277 | 0.012862110939808128 | [0.05076596631955943, 0.08004879536444037] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.270086175943554 | 0.2746368783289175 | 0.04554649174669056 | [0.21708130053455088, 0.32546346558862216] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 76.70782525166447 | 75.36912546435263 | 3.251064657217184 | [73.66775386919305, 80.6720584741697] | 0.0 |
| adaptive | switch_count | 5 | 0 | 421.8 | 422.0 | 235.053610906108 | [154.0, 728.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 4.686656247029354 | 4.688836790702325 | 2.611706776867377 | [1.711111111111111, 8.088888888888889] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 8.579981234776403 | 8.5 | 0.2649834667100048 | [8.255555555555556, 8.9] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.48379799600396856 | 0.4818552433517311 | 0.0032981148803054145 | [0.4808403298999666, 0.4876520251422621] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 9.98123607826384 | 9.967694285714286 | 0.10160791243084492 | [9.878929210526316, 10.122469999999996] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1566.4 | 1553.0 | 28.65833212174079 | [1540.0, 1612.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 465.08150000000006 | 472.053 | 20.40304702428046 | [441.519, 483.7095] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 2033.8 | 1902.0 | 229.92216074141265 | [1840.0, 2365.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 2033.8 | 1902.0 | 229.92216074141265 | [1840.0, 2365.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 5468.0 | 5466.0 | 1070.1296650406437 | [4139.0, 6562.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.1221926877508039 | 0.12323998420920412 | 0.013191828408027554 | [0.10851236126327823, 0.1385672631362443] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[28].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[28].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[38].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[38].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[39].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[39].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[40].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[40].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[41].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[41].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[42].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[42].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[43].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[43].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[44].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[44].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 9018308.195555557 | 9049517.866666667 | 92021.794013469 | [8922480.0, 9134911.644444443] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.059446708058841405 | 0.05617783692531391 | 0.010122558326484342 | [0.046611679962631984, 0.07032704231949803] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.29401921411931864 | 0.2971789666814292 | 0.02184271691079935 | [0.26674406918787197, 0.3218975011758316] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 67.43580710584686 | 74.35632470183114 | 13.560891579870539 | [45.539575662233986, 77.37602965885954] | 0.0 |
| classic | switch_count | 5 | 0 | 4628.2 | 5410.0 | 1420.2276930126382 | [2305.0, 5758.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 51.423977832345074 | 60.110443217297586 | 15.780096848790281 | [25.610826546371708, 63.97706692147865] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 7.591038321796425 | 8.411017655359386 | 1.471278558963643 | [5.2, 8.633237408473239] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.4816240570640417 | 0.4796303519080032 | 0.0038081401098920527 | [0.47875510978652447, 0.4877045855874406] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.015951713058874 | 10.040614935064935 | 0.08785711430481898 | [9.915470263157896, 10.130901923076925] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1593.0 | 1590.0 | 48.176757881783615 | [1536.0, 1666.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 421.01449999999994 | 410.56 | 47.90164733075471 | [369.596, 489.668] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 2000.4 | 2053.0 | 185.85693422630212 | [1749.0, 2244.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 2000.4 | 2053.0 | 185.85693422630212 | [1749.0, 2244.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 4838.0 | 4559.0 | 814.8202255712606 | [3792.0, 5705.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.11222557886484612 | 0.10963044047307428 | 0.013137711602258528 | [0.10178454192778276, 0.1349662614796229] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[28].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[28].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[38].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[38].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[39].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[39].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[40].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[40].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[41].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[41].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[42].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[42].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[43].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[43].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[44].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[44].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 8957737.102222223 | 8928445.866666667 | 75242.56730636326 | [8881069.866666667, 9072328.533333331] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.06724054566349875 | 0.06973942042038388 | 0.007868323461256996 | [0.055492555876650695, 0.07525316765324593] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.27678880906497383 | 0.27113381139802717 | 0.01971824966425882 | [0.2509441853341932, 0.2977684770065179] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 77.02519988224283 | 77.90988270647385 | 3.526950972442411 | [70.8381166208208, 79.26972882426297] | 0.0 |
| edpf | switch_count | 5 | 0 | 1536.2 | 1709.0 | 506.1844525466977 | [650.0, 1865.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 17.06878701347763 | 18.988888888888887 | 5.6242620814028115 | [7.222141976200264, 20.721991977866914] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 8.62216390188257 | 8.755458272685859 | 0.33358342925501155 | [8.033333333333333, 8.833333333333334] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.48303894346853715 | 0.48322403397601765 | 0.003199402541082794 | [0.4778022920880064, 0.4860078705728028] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.066310862593545 | 10.07546493506494 | 0.100720834023555 | [9.95278039473684, 10.21529974358974] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1543.4 | 1546.0 | 10.807404868885037 | [1530.0, 1558.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 487.78329999999994 | 486.991 | 5.903532444647018 | [479.1285, 495.174] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 2364.8 | 2428.0 | 265.1786190476148 | [1928.0, 2651.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 2364.8 | 2428.0 | 265.1786190476148 | [1928.0, 2651.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 5498.0 | 5730.0 | 637.6633124149452 | [4551.0, 6153.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.1357801332767948 | 0.13440815303650488 | 0.007070383069675805 | [0.12575803609406333, 0.14443215470796017] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[28].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[28].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[38].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[38].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[39].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[39].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[40].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[40].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[41].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[41].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[42].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[42].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[43].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[43].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[44].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[44].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 8934084.195555557 | 8914759.466666667 | 147645.0350480984 | [8780819.91111111, 9157254.4] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.06804425108272767 | 0.06994020210950258 | 0.016321422272484648 | [0.04367872773223202, 0.08492665592838963] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.26240091489355477 | 0.2605065306465887 | 0.016399819724548084 | [0.24126205723280192, 0.2827693294260628] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 70.90111485042225 | 77.3748052717449 | 14.850281450425179 | [45.00301727922005, 80.68089900670779] | 0.0 |
| enhanced | switch_count | 5 | 0 | 622.8 | 705.0 | 145.71787810697768 | [419.0, 740.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 6.919941185838676 | 7.833246297263364 | 1.6190864758254033 | [4.655503827735247, 8.222130865212609] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 7.90882017360301 | 8.622222222222222 | 1.612049880471418 | [5.055499383340185, 8.855457161587093] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.4839617922540212 | 0.4828053787336623 | 0.005607718264864357 | [0.47706518576301826, 0.4913947145678548] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.01589829190704 | 10.007863552631584 | 0.15490374536660095 | [9.821858933333331, 10.226131666666667] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1561.8 | 1562.0 | 23.306651411131543 | [1542.0, 1599.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 459.9751 | 481.925 | 37.71055251252625 | [408.146, 489.6185] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 2243.6 | 2150.0 | 222.74379901581995 | [2030.0, 2562.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 2243.6 | 2150.0 | 222.74379901581995 | [2030.0, 2562.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 5545.4 | 5696.0 | 1339.9098103976999 | [3543.0, 6907.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.12235029858505335 | 0.12378193310508297 | 0.011483304188497525 | [0.10880489403539435, 0.13665024749779128] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[28].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[28].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[38].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[38].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[39].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[39].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[40].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[40].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[41].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[41].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[42].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[42].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[43].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[43].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[44].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[44].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 8843075.484444445 | 8843403.022222223 | 103840.6732412577 | [8699520.355555555, 8992198.755555555] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.07882872055265347 | 0.0790684617168524 | 0.010569418362089744 | [0.06351773571034736, 0.09330899042731668] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.2855430510041977 | 0.2859740566648997 | 0.02416608523949704 | [0.256700455551793, 0.3117741563481503] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 68.8558813603287 | 78.02636869497735 | 17.99068204449228 | [37.85997770946858, 80.34542329915787] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 529.8 | 553.0 | 75.51622342252027 | [396.0, 576.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 5.886638790433194 | 6.144444444444445 | 0.8390530302027933 | [4.4, 6.399928889679003] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 7.595519062133877 | 8.633333333333333 | 1.9362942350060117 | [4.2555555555555555, 8.866568149242786] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.4874786576324868 | 0.48714948207862 | 0.0036501686095232484 | [0.483231650260134, 0.4930677341466514] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 9.856947318726792 | 9.906106315789472 | 0.10365764122503858 | [9.709874324324328, 9.966977532467531] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1546.6 | 1541.0 | 17.126003620226175 | [1530.0, 1575.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 484.323 | 481.686 | 4.419599925332606 | [480.471, 490.243] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 1979.4 | 1970.0 | 130.32766398581694 | [1839.0, 2169.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 1979.4 | 1970.0 | 130.32766398581694 | [1839.0, 2169.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 6437.0 | 6478.0 | 846.4685463736972 | [5184.0, 7564.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.12474902129554617 | 0.12133214675173289 | 0.008578905801126217 | [0.11588744431970448, 0.1357824968912605] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[28].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[28].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[38].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[38].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[39].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[39].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[40].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[40].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[41].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[41].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[42].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[42].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[43].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[43].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[44].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[44].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[2] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[3] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[4] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[5] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[6] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[7] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[8] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[9] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[10] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[11] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[12] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[13] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[14] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[15] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[16] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[17] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[18] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[19] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[20] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[21] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[22] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[23] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[24] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[25] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[26] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[27] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[28] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[29] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[30] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[31] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[32] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[33] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[34] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[35] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[36] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[37] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[38] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[39] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[40] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[41] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[42] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[43] graded=False | — | 0.0 | 1.0 |
| adaptive | episode[44] graded=False | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=False | — | 0.0 | 1.0 |
| classic | episode[2] graded=False | — | 0.0 | 1.0 |
| classic | episode[3] graded=False | — | 0.0 | 1.0 |
| classic | episode[4] graded=False | — | 0.0 | 1.0 |
| classic | episode[5] graded=False | — | 0.0 | 1.0 |
| classic | episode[6] graded=False | — | 0.0 | 1.0 |
| classic | episode[7] graded=False | — | 0.0 | 1.0 |
| classic | episode[8] graded=False | — | 0.0 | 1.0 |
| classic | episode[9] graded=False | — | 0.0 | 1.0 |
| classic | episode[10] graded=False | — | 0.0 | 1.0 |
| classic | episode[11] graded=False | — | 0.0 | 1.0 |
| classic | episode[12] graded=False | — | 0.0 | 1.0 |
| classic | episode[13] graded=False | — | 0.0 | 1.0 |
| classic | episode[14] graded=False | — | 0.0 | 1.0 |
| classic | episode[15] graded=False | — | 0.0 | 1.0 |
| classic | episode[16] graded=False | — | 0.0 | 1.0 |
| classic | episode[17] graded=False | — | 0.0 | 1.0 |
| classic | episode[18] graded=False | — | 0.0 | 1.0 |
| classic | episode[19] graded=False | — | 0.0 | 1.0 |
| classic | episode[20] graded=False | — | 0.0 | 1.0 |
| classic | episode[21] graded=False | — | 0.0 | 1.0 |
| classic | episode[22] graded=False | — | 0.0 | 1.0 |
| classic | episode[23] graded=False | — | 0.0 | 1.0 |
| classic | episode[24] graded=False | — | 0.0 | 1.0 |
| classic | episode[25] graded=False | — | 0.0 | 1.0 |
| classic | episode[26] graded=False | — | 0.0 | 1.0 |
| classic | episode[27] graded=False | — | 0.0 | 1.0 |
| classic | episode[28] graded=False | — | 0.0 | 1.0 |
| classic | episode[29] graded=False | — | 0.0 | 1.0 |
| classic | episode[30] graded=False | — | 0.0 | 1.0 |
| classic | episode[31] graded=False | — | 0.0 | 1.0 |
| classic | episode[32] graded=False | — | 0.0 | 1.0 |
| classic | episode[33] graded=False | — | 0.0 | 1.0 |
| classic | episode[34] graded=False | — | 0.0 | 1.0 |
| classic | episode[35] graded=False | — | 0.0 | 1.0 |
| classic | episode[36] graded=False | — | 0.0 | 1.0 |
| classic | episode[37] graded=False | — | 0.0 | 1.0 |
| classic | episode[38] graded=False | — | 0.0 | 1.0 |
| classic | episode[39] graded=False | — | 0.0 | 1.0 |
| classic | episode[40] graded=False | — | 0.0 | 1.0 |
| classic | episode[41] graded=False | — | 0.0 | 1.0 |
| classic | episode[42] graded=False | — | 0.0 | 1.0 |
| classic | episode[43] graded=False | — | 0.0 | 1.0 |
| classic | episode[44] graded=False | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=False | — | 0.0 | 1.0 |
| edpf | episode[2] graded=False | — | 0.0 | 1.0 |
| edpf | episode[3] graded=False | — | 0.0 | 1.0 |
| edpf | episode[4] graded=False | — | 0.0 | 1.0 |
| edpf | episode[5] graded=False | — | 0.0 | 1.0 |
| edpf | episode[6] graded=False | — | 0.0 | 1.0 |
| edpf | episode[7] graded=False | — | 0.0 | 1.0 |
| edpf | episode[8] graded=False | — | 0.0 | 1.0 |
| edpf | episode[9] graded=False | — | 0.0 | 1.0 |
| edpf | episode[10] graded=False | — | 0.0 | 1.0 |
| edpf | episode[11] graded=False | — | 0.0 | 1.0 |
| edpf | episode[12] graded=False | — | 0.0 | 1.0 |
| edpf | episode[13] graded=False | — | 0.0 | 1.0 |
| edpf | episode[14] graded=False | — | 0.0 | 1.0 |
| edpf | episode[15] graded=False | — | 0.0 | 1.0 |
| edpf | episode[16] graded=False | — | 0.0 | 1.0 |
| edpf | episode[17] graded=False | — | 0.0 | 1.0 |
| edpf | episode[18] graded=False | — | 0.0 | 1.0 |
| edpf | episode[19] graded=False | — | 0.0 | 1.0 |
| edpf | episode[20] graded=False | — | 0.0 | 1.0 |
| edpf | episode[21] graded=False | — | 0.0 | 1.0 |
| edpf | episode[22] graded=False | — | 0.0 | 1.0 |
| edpf | episode[23] graded=False | — | 0.0 | 1.0 |
| edpf | episode[24] graded=False | — | 0.0 | 1.0 |
| edpf | episode[25] graded=False | — | 0.0 | 1.0 |
| edpf | episode[26] graded=False | — | 0.0 | 1.0 |
| edpf | episode[27] graded=False | — | 0.0 | 1.0 |
| edpf | episode[28] graded=False | — | 0.0 | 1.0 |
| edpf | episode[29] graded=False | — | 0.0 | 1.0 |
| edpf | episode[30] graded=False | — | 0.0 | 1.0 |
| edpf | episode[31] graded=False | — | 0.0 | 1.0 |
| edpf | episode[32] graded=False | — | 0.0 | 1.0 |
| edpf | episode[33] graded=False | — | 0.0 | 1.0 |
| edpf | episode[34] graded=False | — | 0.0 | 1.0 |
| edpf | episode[35] graded=False | — | 0.0 | 1.0 |
| edpf | episode[36] graded=False | — | 0.0 | 1.0 |
| edpf | episode[37] graded=False | — | 0.0 | 1.0 |
| edpf | episode[38] graded=False | — | 0.0 | 1.0 |
| edpf | episode[39] graded=False | — | 0.0 | 1.0 |
| edpf | episode[40] graded=False | — | 0.0 | 1.0 |
| edpf | episode[41] graded=False | — | 0.0 | 1.0 |
| edpf | episode[42] graded=False | — | 0.0 | 1.0 |
| edpf | episode[43] graded=False | — | 0.0 | 1.0 |
| edpf | episode[44] graded=False | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[2] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[4] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[6] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[7] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[8] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[9] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[10] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[11] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[12] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[13] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[14] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[15] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[16] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[17] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[18] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[19] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[20] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[21] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[22] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[23] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[24] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[25] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[26] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[27] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[28] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[29] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[30] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[31] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[32] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[33] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[34] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[35] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[36] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[37] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[38] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[39] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[40] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[41] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[42] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[43] graded=False | — | 0.0 | 1.0 |
| enhanced | episode[44] graded=False | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[2] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[3] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[4] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[5] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[6] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[7] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[8] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[9] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[10] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[11] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[12] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[13] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[14] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[15] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[16] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[17] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[18] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[19] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[20] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[21] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[22] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[23] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[24] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[25] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[26] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[27] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[28] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[29] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[30] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[31] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[32] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[33] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[34] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[35] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[36] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[37] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[38] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[39] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[40] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[41] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[42] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[43] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | episode[44] graded=False | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9761637000555835, 1.0047853162897409] | 1.0544166932013785 | None | () |
| adaptive | edpf | 5 | [0.9816930363276776, 1.021974994424255] | -0.301741656305618 | None | () |
| adaptive | enhanced | 5 | [0.9790245522597786, 1.0377544495363957] | -0.32181982521748836 | None | () |
| adaptive | rtt-threshold | 5 | [0.9989153295678513, 1.0321443714225143] | -1.2346457859524707 | None | () |
| classic | adaptive | 5 | [0.9952374739039667, 1.0244183428896805] | -1.0544166932013785 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.9928437597585094, 1.0245070385579154] | -1.3561583495069967 | None | () |
| classic | enhanced | 5 | [0.9743619222809841, 1.0403255888308642] | -1.376236518418867 | None | () |
| classic | rtt-threshold | 5 | [1.0073759935476319, 1.0347016111935552] | -2.2890624791538494 | None | () |
| edpf | adaptive | 5 | [0.9784975224010886, 1.018648358493817] | 0.301741656305618 | None | () |
| edpf | classic | 5 | [0.9760791896633416, 1.0072078211411948] | 1.3561583495069967 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.9750134130451444, 1.0181085986037701] | -0.020078168911870364 | None | () |
| edpf | rtt-threshold | 5 | [1.0042593155994126, 1.0334413532520272] | -0.9329041296468527 | None | () |
| enhanced | adaptive | 5 | [0.9636190916326478, 1.0214248434237998] | 0.32181982521748836 | None | () |
| enhanced | classic | 5 | [0.9612375305733056, 1.0263126843657817] | 1.376236518418867 | None | () |
| enhanced | edpf | 5 | [0.9822134901634225, 1.0256269161229463] | 0.020078168911870364 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.9945940216239135, 1.0345183031584513] | -0.9128259607349822 | None | () |
| rtt-threshold | adaptive | 5 | [0.9688567100567408, 1.0010858482196063] | 1.2346457859524707 | None | () |
| rtt-threshold | classic | 5 | [0.9664622043513338, 0.9926780133784446] | 2.2890624791538494 | None | () |
| rtt-threshold | edpf | 5 | [0.9676407827625688, 0.9957587492261694] | 0.9329041296468527 | None | () |
| rtt-threshold | enhanced | 5 | [0.9666334534120231, 1.0054353618245764] | 0.9128259607349822 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@baseline--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| upstream-classic | useful_goodput_bps | 5 | 0 | 9581696.568888888 | 9580480.0 | 10375.671719223097 | [9569601.066666666, 9592645.688888889] | 0.0 |
| upstream-classic | viewer_loss_ratio | 5 | 0 | 0.0017056245229299042 | 0.001837787838956234 | 0.0010279198357230684 | [0.0002924225993932231, 0.0028933356026161587] | 0.0 |
| upstream-classic | per_link_share_gini | 5 | 0 | 0.31800918304006565 | 0.32374927074398296 | 0.015111603505644297 | [0.29164984818678763, 0.32995287142348995] | 0.0 |
| upstream-classic | cpu_ms_per_mb | 5 | 0 | 42.84118276196965 | 37.576405357560375 | 8.266982398394784 | [36.302289967020066, 52.63740652087761] | 0.0 |
| upstream-classic | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | sender_cpu_percent | 5 | 0 | 5.131077432472972 | 4.4999500005555495 | 0.9900521548914355 | [4.344396173375851, 6.3110409884334615] | 0.0 |
| upstream-classic | diagnostics.loss_ratio | 5 | 0 | 0.4568554250184601 | 0.4568560029562265 | 0.0009978927314748993 | [0.4556156638061269, 0.4581696274712536] | 0.0 |
| upstream-classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.468589853658537 | 10.470486951219511 | 0.024373694878404583 | [10.43175695121951, 10.498445975609751] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1557.6 | 1565.0 | 11.081516141756055 | [1545.0, 1567.0] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-classic | diagnostics.ms_rtt_median | 5 | 0 | 59.7855 | 59.543 | 1.1123558895425507 | [58.91, 61.6395] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_delta | 5 | 0 | 3112.2 | 3201.0 | 287.1266271177231 | [2667.0, 3419.0] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_sum | 5 | 0 | 3112.2 | 3201.0 | 287.1266271177231 | [2667.0, 3419.0] | 0.0 |
| upstream-classic | diagnostics.pkt_drop_delta | 5 | 0 | 140.2 | 151.0 | 84.58841528247234 | [24.0, 238.0] | 0.0 |
| upstream-classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | diagnostics.retrans_ratio | 5 | 0 | 0.06719366770395696 | 0.06735470479704797 | 0.003641563021160603 | [0.06332024029574862, 0.07085118142876883] | 0.0 |
| upstream-classic | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[28].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[28].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[38].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[38].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[39].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[39].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[40].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[40].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[41].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[41].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[42].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[42].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[43].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[43].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[44].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[44].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| upstream-enhanced | useful_goodput_bps | 5 | 0 | 9563214.08 | 9573110.4 | 32295.888750445156 | [9506784.0, 9586328.888888888] | 0.0 |
| upstream-enhanced | viewer_loss_ratio | 5 | 0 | 0.0038051061997496577 | 0.00287013839904653 | 0.0032040164541294316 | [0.0016318776335338676, 0.009427617661804086] | 0.0 |
| upstream-enhanced | per_link_share_gini | 5 | 0 | 0.2939099379667753 | 0.307805534269616 | 0.03392480687707417 | [0.23603000750167716, 0.317135684664964] | 0.0 |
| upstream-enhanced | cpu_ms_per_mb | 5 | 0 | 45.0162263693404 | 46.38665922649392 | 6.303071112280844 | [37.77419483929698, 51.53323347585476] | 0.0 |
| upstream-enhanced | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | sender_cpu_percent | 5 | 0 | 5.382186099166799 | 5.555493827846357 | 0.7615151284000351 | [4.488888888888889, 6.166666666666667] | 0.0 |
| upstream-enhanced | diagnostics.loss_ratio | 5 | 0 | 0.4577418058921271 | 0.45872575830585055 | 0.0019088300384083844 | [0.45495800301707817, 0.4593168231201904] | 0.0 |
| upstream-enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.461493060825054 | 10.428648414634148 | 0.10669763290472273 | [10.332169878048784, 10.60571256097561] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1554.2 | 1558.0 | 10.825894882179487 | [1537.0, 1566.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rtt_median | 5 | 0 | 65.4958 | 62.7105 | 6.044488032497042 | [60.2705, 72.82] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 3651.8 | 3387.0 | 695.5894622548562 | [2819.0, 4406.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 3651.8 | 3387.0 | 695.5894622548562 | [2819.0, 4406.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 312.0 | 236.0 | 261.65721851307677 | [134.0, 771.0] | 0.0 |
| upstream-enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.06828353807775157 | 0.06543806506809721 | 0.013417452910099483 | [0.051197926396413226, 0.08379405666897029] | 0.0 |
| upstream-enhanced | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[7].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[7].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[8].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[8].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[9].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[9].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[10].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[10].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[11].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[11].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[12].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[12].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[13].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[13].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[14].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[14].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[15].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[15].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[16].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[16].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[17].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[17].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[18].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[18].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[19].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[19].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[20].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[20].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[21].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[21].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[22].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[22].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[23].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[23].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[24].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[24].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[25].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[25].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[26].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[26].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[27].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[27].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[28].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[28].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[29].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[29].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[30].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[30].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[31].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[31].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[32].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[32].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[33].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[33].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[34].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[34].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[35].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[35].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[36].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[36].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[37].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[37].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[38].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[38].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[39].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[39].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[40].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[40].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[41].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[41].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[42].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[42].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[43].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[43].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[44].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[44].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| upstream-classic | episode[1] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[2] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[3] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[4] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[5] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[6] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[7] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[8] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[9] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[10] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[11] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[12] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[13] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[14] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[15] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[16] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[17] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[18] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[19] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[20] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[21] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[22] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[23] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[24] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[25] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[26] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[27] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[28] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[29] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[30] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[31] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[32] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[33] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[34] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[35] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[36] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[37] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[38] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[39] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[40] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[41] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[42] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[43] graded=False | — | 0.0 | 1.0 |
| upstream-classic | episode[44] graded=False | — | 0.0 | 1.0 |
| upstream-classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-classic | settled_rate | — | 1.0 | — |
| upstream-classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| upstream-enhanced | episode[1] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[2] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[3] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[4] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[5] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[6] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[7] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[8] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[9] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[10] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[11] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[12] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[13] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[14] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[15] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[16] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[17] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[18] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[19] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[20] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[21] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[22] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[23] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[24] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[25] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[26] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[27] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[28] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[29] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[30] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[31] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[32] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[33] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[34] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[35] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[36] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[37] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[38] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[39] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[40] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[41] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[42] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[43] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | episode[44] graded=False | — | 0.0 | 1.0 |
| upstream-enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-enhanced | settled_rate | — | 1.0 | — |
| upstream-enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| upstream-classic | upstream-classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| upstream-classic | upstream-enhanced | 5 | [0.998706528370958, 1.006607604282023] | -0.10323505600902962 | None | () |
| upstream-enhanced | upstream-classic | 5 | [0.9934357695551725, 1.001295146864767] | 0.10323505600902962 | None | () |
| upstream-enhanced | upstream-enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M2@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 7877576.0 | 7933725.333333333 | 78465.68381825717 | [7787561.6, 7936006.4] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.01412076257365645 | 0.007162218745179276 | 0.010112738720357926 | [0.00630233583076245, 0.025965573733701383] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.12129022392035234 | 0.010207678819031174 | 0.1589797152863574 | [0.0021763461434897158, 0.29720102414104177] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 37.731611113385355 | 39.32578793585661 | 17.883315359477184 | [17.95836819252137, 64.862137040051] | 0.0 |
| adaptive | switch_count | 5 | 0 | 1108.0 | 1115.0 | 84.69061341140468 | [1018.0, 1232.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 18.46653522441293 | 18.583333333333332 | 1.4113607869711484 | [16.966666666666665, 20.53299111681472] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 3.7266317783592497 | 3.9 | 1.7903088284898594 | [1.75, 6.433226112898119] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.3182020983359795 | 0.3448898053872922 | 0.04065350242368826 | [0.2723490311137633, 0.34937672146974474] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.21622551212121 | 8.328927555555556 | 0.16585505132280964 | [8.034350454545455, 8.343670666666668] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1374.0 | 1377.0 | 14.474114826130128 | [1358.0, 1390.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 212.1362 | 30.681 | 250.8624165896518 | [27.609, 489.177] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 400.4 | 408.0 | 55.21141186385292 | [323.0, 470.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 400.4 | 408.0 | 55.21141186385292 | [323.0, 470.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 632.0 | 325.0 | 446.89763928667156 | [286.0, 1157.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.04827447765102818 | 0.035697120636169005 | 0.0174435349055039 | [0.0354820407628857, 0.06885769713568413] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | 1790.4 | 0.0 | 4003.456106915623 | [0.0, 8952.0] | 0.0 |
| adaptive | episode[1].recovery_ms | 5 | 0 | 1884.0 | 1930.0 | 79.84985911070852 | [1795.0, 1953.0] | 0.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 7852870.293333334 | 7801072.533333333 | 79220.27550157688 | [7790895.466666667, 7942849.6] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.01772052487190171 | 0.02439298814104728 | 0.009924066227968275 | [0.006788028386300524, 0.025367936186945286] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.17008247755096167 | 0.2506172841116674 | 0.11331013109020883 | [0.04292563230122312, 0.2570160272522183] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 36.625437315005136 | 40.65767041225735 | 16.2101813984332 | [19.48450028512319, 54.74751643024294] | 0.0 |
| classic | switch_count | 5 | 0 | 9419.8 | 9346.0 | 161.8137818605078 | [9235.0, 9612.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 156.99666666666667 | 155.76666666666668 | 2.696896364341791 | [153.91666666666666, 160.2] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 3.59 | 4.033333333333333 | 1.573636976349162 | [1.9, 5.333333333333333] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.28624195117427637 | 0.2572595127570901 | 0.04210920974479141 | [0.25402498383198885, 0.332701780977643] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.160062054545454 | 8.032826136363637 | 0.1763131771206148 | [8.029208863636363, 8.359034000000003] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1374.8 | 1372.0 | 18.019433953373785 | [1357.0, 1404.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 317.0314 | 511.918 | 276.25955915438476 | [14.124, 524.8165] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 425.2 | 349.0 | 129.73318773544418 | [318.0, 620.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 425.2 | 349.0 | 129.73318773544418 | [318.0, 620.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 790.6 | 1084.0 | 438.13217183859024 | [308.0, 1129.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.05525340519273088 | 0.06645829530936302 | 0.017386311879774557 | [0.035119905648014677, 0.06902214821742204] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | 5394.0 | 0.0 | 7789.869575288151 | [0.0, 16986.0] | 0.0 |
| classic | episode[1].recovery_ms | 5 | 0 | 1880.4 | 1909.0 | 82.42754394011749 | [1788.0, 1958.0] | 0.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 7878488.426666667 | 7927934.933333334 | 81974.43237554276 | [7787035.2, 7943376.0] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.014334856087849821 | 0.007860665844636251 | 0.009750789261265751 | [0.006510704038843522, 0.025713516424340332] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.20067705152082974 | 0.13775708034923376 | 0.12909627185411351 | [0.08601689471614207, 0.34869288310822544] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 45.40029602892115 | 44.81721625666467 | 12.582315750895457 | [28.423312293918144, 59.53631102791426] | 0.0 |
| edpf | switch_count | 5 | 0 | 2806.0 | 2827.0 | 615.6849843873082 | [2173.0, 3672.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 46.76613411998689 | 47.11588140197664 | 10.260953204546388 | [36.21666666666667, 61.19898001699972] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 4.473286000788876 | 4.449925834569424 | 1.2445573287355016 | [2.7666666666666666, 5.899901668305528] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.3245930027467622 | 0.3539886642416039 | 0.04128868846017495 | [0.27883809241998325, 0.35603088216681433] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.217842888888889 | 8.324370444444444 | 0.16408316386095528 | [8.036297045454544, 8.351102444444445] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1356.2 | 1339.0 | 36.39642839620394 | [1337.0, 1421.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 203.03099999999998 | 15.074 | 257.64312058058135 | [14.797, 495.253] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 390.8 | 365.0 | 108.91143190684805 | [242.0, 511.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 390.8 | 365.0 | 108.91143190684805 | [242.0, 511.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 641.8 | 357.0 | 430.90567413298237 | [295.0, 1146.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.051058287058811304 | 0.03747379454926625 | 0.01968801294529189 | [0.03622951177459133, 0.0733547607650049] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | 1951.0 | None | [1787.0, +inf] | 0.2 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 7824760.533333333 | 7796334.933333334 | 66261.38047648035 | [7791597.333333333, 7943200.533333333] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.02091100579250604 | 0.02427162606385374 | 0.008079367985768011 | [0.006486915846608711, 0.02532811938151744] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.24424208229597424 | 0.2943373345606962 | 0.1140727178720568 | [0.040287994920048115, 0.2999752304818408] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 42.552855738203434 | 45.48964007858193 | 10.975327514180806 | [23.61518339927902, 52.00545701471843] | 0.0 |
| enhanced | switch_count | 5 | 0 | 1089.6 | 1025.0 | 158.35182348176482 | [1004.0, 1372.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 18.15988727965645 | 17.083048615856402 | 2.6392637884971664 | [16.733054449092513, 22.866666666666667] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 4.1633112225907345 | 4.516666666666667 | 1.076169450727168 | [2.2999616673055447, 5.066666666666666] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.2907242146626385 | 0.276423301034978 | 0.033103450864514224 | [0.27302724303172, 0.3497258420565406] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.095427691919191 | 8.038430454545455 | 0.1286916070628246 | [8.035114318181817, 8.325617777777778] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1374.6 | 1369.0 | 16.10279478848315 | [1361.0, 1401.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 381.57989999999995 | 459.7005 | 194.80888857781875 | [35.532, 490.253] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 310.4 | 338.0 | 84.16234312327575 | [165.0, 368.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 310.4 | 338.0 | 84.16234312327575 | [165.0, 368.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 930.8 | 1078.0 | 356.7319722144344 | [294.0, 1127.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.06342801635958815 | 0.07017223679707996 | 0.0167931716092767 | [0.03364068420245735, 0.07433095297848555] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | 1789.6 | 0.0 | 4001.6672525336235 | [0.0, 8948.0] | 0.0 |
| enhanced | episode[1].recovery_ms | 5 | 0 | 1834.4 | 1796.0 | 74.74155470686972 | [1773.0, 1950.0] | 0.0 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 7912704.426666667 | 7934953.6 | 59474.96957808932 | [7806687.466666667, 7946709.866666666] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.010051568014455633 | 0.00672309659216153 | 0.0077357066492303094 | [0.006154864328259431, 0.02388140404632091] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.1504765916299247 | 0.15041454233946028 | 0.0021327725767484205 | [0.14736535407527243, 0.15282667863244484] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 41.40415773154415 | 42.17626006869992 | 7.964628940477293 | [28.019981904796357, 48.56473235186194] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 15.0 | 18.0 | 6.855654600401044 | [3.0, 20.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 0.2499990000166664 | 0.29999500008333196 | 0.11426036304265222 | [0.05, 0.3333333333333333] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 4.093319389121293 | 4.183333333333334 | 0.7779582802096255 | [2.783333333333333, 4.816666666666666] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.3155236799322069 | 0.3298429319371728 | 0.03266039853654978 | [0.25710842146361335, 0.331084443143568] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.268058965656564 | 8.324441555555554 | 0.13131248678588123 | [8.033317272727274, 8.334265999999998] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1339.4 | 1339.0 | 3.2093613071762426 | [1335.0, 1343.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 107.1783 | 14.962 | 206.5620599709685 | [14.462, 476.6875] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 351.6 | 406.0 | 110.17622248017037 | [168.0, 432.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 351.6 | 406.0 | 110.17622248017037 | [168.0, 432.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 451.2 | 305.0 | 340.5439766021416 | [279.0, 1060.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.04135230323891172 | 0.03612138184523159 | 0.015409673808662164 | [0.03239859022350649, 0.06871101158512899] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.2 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | 1946.0 | None | [1713.0, +inf] | 0.2 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[2] graded=True | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 1.0 | 0.0 |
| classic | episode[2] graded=True | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 0.8 | 0.19999999999999996 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 0.8 | 0.19999999999999996 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [0.9812730770081143, 1.0186257066282292] | -1.7230769395868004 | 1.0061521252796422 | () |
| adaptive | edpf | 5 | [0.9804060173628752, 1.0184680525213397] | -0.06984470994569753 | 0.9220912352639672 | () |
| adaptive | enhanced | 5 | [0.99909430294462, 1.0180553366802496] | -1.710940731867446 | 1.0124083474337282 | () |
| adaptive | rtt-threshold | 5 | [0.9799730618914085, 1.0164078128160752] | 0.043912215301774526 | 1.0 | () |
| classic | adaptive | 5 | [0.9817148668966127, 1.019084313460412] | 1.7230769395868004 | 0.9938854919399667 | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| classic | edpf | 5 | [0.9827143552742242, 1.0018026544086167] | 1.653232229641103 | 0.9164531009738596 | () |
| classic | enhanced | 5 | [0.9808257306324416, 1.0187927619733526] | 0.012136207719353961 | 1.0022421524663676 | () |
| classic | rtt-threshold | 5 | [0.9819102589619407, 1.0009950908849674] | 1.766989154888575 | 0.9713513513513513 | () |
| edpf | adaptive | 5 | [0.9818668317816943, 1.019985579739534] | 0.06984470994569753 | 1.084491384102279 | () |
| edpf | classic | 5 | [0.998200589307002, 1.0175896939258124] | -1.653232229641103 | 1.0911633109619687 | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| edpf | enhanced | 5 | [0.9980781550288277, 1.019457268325639] | -1.6410960219217487 | 1.093609865470852 | () |
| edpf | rtt-threshold | 5 | [0.9806430228704011, 1.0010614302772987] | 0.11375692524747205 | 1.0431990659661412 | () |
| enhanced | adaptive | 5 | [0.9822648769377058, 1.0009065180861412] | 1.710940731867446 | 0.9877437325905293 | () |
| enhanced | classic | 5 | [0.9815538913557338, 1.019549109254296] | -0.012136207719353961 | 0.9977628635346756 | () |
| enhanced | edpf | 5 | [0.9809140913207713, 1.0019255455712452] | 1.6410960219217487 | 0.9144028703229113 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [0.9804809114796087, 1.0011057298923018] | 1.754852947169221 | 0.956989247311828 | () |
| rtt-threshold | adaptive | 5 | [0.9838570575617522, 1.0204362128790951] | -0.043912215301774526 | 1.0 | () |
| rtt-threshold | classic | 5 | [0.9990058983365365, 1.0184230084907997] | -1.766989154888575 | 1.0294936004451865 | () |
| rtt-threshold | edpf | 5 | [0.9989396951623591, 1.0197390657743528] | -0.11375692524747205 | 0.9974372116863147 | () |
| rtt-threshold | enhanced | 5 | [0.998895491395878, 1.0199076680553991] | -1.754852947169221 | 1.0449438202247192 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M3@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 8000086.826666666 | 8000227.2 | 436.90847655906043 | [7999349.866666666, 8000402.666666667] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.06333701928681154 | 0.06505501972225577 | 0.006501685560407805 | [0.0536533842109963, 0.07129053951216854] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 31.73270926027929 | 30.166471590150383 | 12.438763220221636 | [18.33482334331037, 51.49853744153666] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2558.2 | 2526.0 | 71.49965034879541 | [2499.0, 2676.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 42.63624506258229 | 42.1 | 1.1918681912528484 | [41.649305844902585, 44.6] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 3.173300000555546 | 3.016616389726838 | 1.2439389154912048 | [1.8333027782870288, 5.149914168097198] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.2758600158554779 | 0.2739384783517084 | 0.010250451754055355 | [0.2659227822548599, 0.2875432905520819] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.326797625120772 | 8.32205688888889 | 0.02771420447971925 | [8.300224347826086, 8.357082666666667] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1725.8 | 1700.0 | 53.68146793820005 | [1679.0, 1793.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 17.479300000000002 | 16.649 | 2.1535374851624938 | [16.096, 21.2795] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 270.4 | 202.0 | 132.22443042040302 | [158.0, 447.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 270.4 | 202.0 | 132.22443042040302 | [158.0, 447.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.0241569230676028 | 0.02015630519251148 | 0.012462943546258636 | [0.010934759543554988, 0.03738153352243991] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[1].recovery_ms | 5 | 0 | 1962.6 | 1985.0 | 41.18009227770137 | [1894.0, 1990.0] | 0.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 7968081.706666666 | 7961449.066666666 | 18542.435330020504 | [7951447.466666667, 7999349.866666666] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.00390836741157193 | 0.004622666548703884 | 0.0023322464879462984 | [2.1738185296291465e-05, 0.006095137140585663] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.2599997339271622 | 0.27748367670160556 | 0.044583067322810914 | [0.18233123302220752, 0.28913095936526506] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 45.39937728884398 | 36.33628626219691 | 18.042125697881385 | [28.7788369802701, 73.78111584413243] | 0.0 |
| classic | switch_count | 5 | 0 | 16062.2 | 15257.0 | 2220.224921038407 | [14502.0, 19977.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 267.7024970139386 | 254.28333333333333 | 37.00422344203053 | [241.7, 332.95] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 4.519975555962956 | 3.6333333333333337 | 1.7892100280211538 | [2.8666666666666667, 7.333211113148114] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.15208378205717754 | 0.14117032587811748 | 0.03040908602391677 | [0.13206908228919742, 0.20564205333744412] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.382305642512076 | 8.389902888888889 | 0.03376161040765828 | [8.33349043478261, 8.418728444444442] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1429.8 | 1417.0 | 44.40382866375376 | [1399.0, 1508.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 6.8086 | 6.805 | 0.22280327645705766 | [6.587, 7.121] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 680.8 | 728.0 | 251.70359552457725 | [276.0, 949.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 680.8 | 728.0 | 251.70359552457725 | [276.0, 949.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 176.8 | 209.0 | 105.55661987767512 | [1.0, 276.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.03564728026722922 | 0.03939532753092075 | 0.006283660777603927 | [0.025486771459990943, 0.040295660467334286] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[1].recovery_ms | 5 | 0 | 1959.8 | 1948.0 | 24.893774322107124 | [1937.0, 1991.0] | 0.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 7845956.906666666 | 7840201.6 | 58358.74669634346 | [7761943.466666667, 7924425.6] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.020362127701069092 | 0.02126285790622078 | 0.007136078232400015 | [0.010473738640614344, 0.030345769883421116] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.07976467792384212 | 0.07995819035292318 | 0.0022374168740551138 | [0.0769197438900373, 0.0829792341945983] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 51.30140253384311 | 46.25731392757383 | 21.49364011098174 | [27.466710685744072, 86.06092055020721] | 0.0 |
| edpf | switch_count | 5 | 0 | 3933.4 | 3968.0 | 188.25461481727348 | [3635.0, 4154.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 65.55624672922119 | 66.13333333333334 | 3.137990663307035 | [60.58232362793954, 69.23333333333333] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 5.023290389604618 | 4.533257779037016 | 2.0709655113448138 | [2.7, 8.34986083565274] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.2855801713686248 | 0.28636277061773857 | 0.006007691581987151 | [0.27687543449409086, 0.2935749639746883] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.27213706161616 | 8.259487045454545 | 0.03127800617494109 | [8.247809545454547, 8.324076444444445] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1412.0 | 1401.0 | 20.43281674170255 | [1396.0, 1441.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 11.7164 | 11.77 | 0.15120656401095767 | [11.510000000000002, 11.8835] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 669.8 | 674.0 | 190.83422124975385 | [363.0, 864.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 669.8 | 674.0 | 190.83422124975385 | [363.0, 864.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 918.0 | 955.0 | 323.74372580792976 | [476.0, 1377.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.04332776960945915 | 0.04223779970531768 | 0.005283305133567612 | [0.03775563712637651, 0.05099665195893661] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | 7068.2 | 6938.0 | 516.1721612020548 | [6571.0, 7944.0] | 0.0 |
| edpf | episode[1].recovery_ms | 5 | 0 | 2166.8 | 1981.0 | 430.09150189232986 | [1962.0, 2936.0] | 0.0 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 7998753.279999999 | 7999525.333333333 | 1948.3912214954887 | [7995489.6, 8000227.2] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.07273360481792827 | 0.06783400687677663 | 0.015933996716783215 | [0.05550429870498483, 0.0982355852696046] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 43.27304421540552 | 41.49973163506876 | 8.622479672769225 | [35.3354299021742, 56.51078979346457] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2506.6 | 2534.0 | 95.51334985225887 | [2340.0, 2582.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 41.77639589340178 | 42.23262945617573 | 1.5921221473098837 | [38.999350010833155, 43.03333333333333] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 4.32664272262129 | 4.15 | 0.8623221538974636 | [3.5332744454259095, 5.65] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.26238291786402723 | 0.2651681019802462 | 0.01095341554076363 | [0.24404583956153464, 0.27299942058842463] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.318550444444444 | 8.30837888888889 | 0.023696035855375996 | [8.298526888888889, 8.35761888888889] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1687.8 | 1711.0 | 62.838682354104144 | [1583.0, 1746.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 18.0296 | 19.165 | 2.8796383974381237 | [14.056, 21.067] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 203.0 | 218.0 | 52.02403290787826 | [147.0, 272.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 203.0 | 218.0 | 52.02403290787826 | [147.0, 272.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.014828860673758413 | 0.01516255048665828 | 0.0035887977331220556 | [0.010061029541836191, 0.01995575221238938] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[1].recovery_ms | 5 | 0 | 1965.4 | 1960.0 | 21.651789764358973 | [1942.0, 1990.0] | 0.0 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 7825988.8 | 7831077.333333333 | 42992.37703510788 | [7760890.666666667, 7877225.066666666] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.020596938029272784 | 0.02132722501445537 | 0.003788170342359406 | [0.015143557422969188, 0.025603543743078622] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.4388245673328628 | 0.43880290698669994 | 0.00016874615836165512 | [0.43863544862441617, 0.4390449092733076] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 55.92094990122367 | 53.99532573128798 | 12.663223510333665 | [38.577684614274474, 73.35927766366335] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 10.0 | 10.0 | 1.4142135623730951 | [8.0, 12.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 0.16666566668333305 | 0.16666666666666666 | 0.02357101172815532 | [0.13333111114814752, 0.2] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 5.466625222912951 | 5.316578057032383 | 1.2146869358296826 | [3.783333333333333, 7.116548057532374] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.05354127232439336 | 0.053035782666042634 | 0.003923498283921146 | [0.0483928459103013, 0.05938785792257957] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.17030916121212 | 8.164444772727274 | 0.029874217916406764 | [8.141399999999997, 8.206242533333333] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1433.2 | 1409.0 | 44.41508752665022 | [1394.0, 1500.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 15.8 | 15.9075 | 0.6848845705664568 | [14.697, 16.548] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 357.4 | 287.0 | 171.24923357492727 | [262.0, 663.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 357.4 | 287.0 | 171.24923357492727 | [262.0, 663.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 929.0 | 959.0 | 168.0996728134829 | [692.0, 1156.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.04146254748131498 | 0.04054995169950352 | 0.0021531576990696326 | [0.04006204896472651, 0.04526078514517821] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | 5968.8 | 5984.0 | 22.331591971912793 | [5941.0, 5986.0] | 0.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | 1968.2 | 1965.0 | 19.097120201747696 | [1950.0, 1989.0] | 0.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[2] graded=True | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 1.0 | 0.0 |
| classic | episode[2] graded=True | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 1.0 | 0.0 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [1.0001316106955627, 1.0061346985612145] | -0.46226665487038837 | 1.0030800821355237 | () |
| adaptive | edpf | 5 | [1.0095877064788983, 1.0306989782077947] | -2.126285790622078 | 1.0025188916876575 | () |
| adaptive | enhanced | 5 | [0.9999780653652116, 1.0006144797770318] | 0.0 | 1.0061791967044285 | () |
| adaptive | rtt-threshold | 5 | [1.0156371817432563, 1.0308161881076192] | -2.132722501445537 | 1.0015098137896326 | () |
| classic | adaptive | 5 | [0.9939027064964688, 0.9998684066235332] | 0.46226665487038837 | 0.9969293756397134 | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| classic | edpf | 5 | [1.0094548514237633, 1.024414504023872] | -1.6640191357516896 | 0.9852791878172589 | () |
| classic | enhanced | 5 | [0.9941207441207441, 1.0004828055390962] | 0.46226665487038837 | 1.000502512562814 | () |
| classic | rtt-threshold | 5 | [1.0104247878288375, 1.025842188559801] | -1.6704558465751487 | 0.9953846153846154 | () |
| edpf | adaptive | 5 | [0.9702153792165635, 0.9905033446649851] | 2.126285790622078 | 0.9974874371859297 | () |
| edpf | classic | 5 | [0.9761673581075117, 0.9906337054991335] | 1.6640191357516896 | 1.014940752189593 | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| edpf | enhanced | 5 | [0.9704282204282204, 0.991111988939364] | 2.126285790622078 | 1.0010204081632652 | () |
| edpf | rtt-threshold | 5 | [0.9911718574949586, 1.0100836536287587] | -0.006436710823459063 | 1.0102564102564102 | () |
| enhanced | adaptive | 5 | [0.9993858975764885, 1.0000219351159272] | 0.0 | 0.9938587512794268 | () |
| enhanced | classic | 5 | [0.999517427449604, 1.005914025951099] | -0.46226665487038837 | 0.9994977398292315 | () |
| enhanced | edpf | 5 | [1.0089677162216022, 1.0304729179853513] | -2.126285790622078 | 0.9989806320081549 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [1.0155926313679193, 1.030838797196473] | -2.132722501445537 | 0.9994871794871795 | () |
| rtt-threshold | adaptive | 5 | [0.9701050599872788, 0.9846035749533939] | 2.132722501445537 | 0.9984924623115577 | () |
| rtt-threshold | classic | 5 | [0.9748088069997576, 0.9896827671347633] | 1.6704558465751487 | 1.0046367851622875 | () |
| rtt-threshold | edpf | 5 | [0.9900170113707584, 1.0089067727642644] | 0.006436710823459063 | 0.9898477157360406 | () |
| rtt-threshold | enhanced | 5 | [0.9700837829538974, 0.9846467659509135] | 2.132722501445537 | 1.0005130836326321 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M4@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 7999244.586666666 | 7999174.4 | 266.1083755829461 | [7998998.933333334, 7999525.333333333] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.12726704129312313 | 0.12576805632410806 | 0.009507378860528468 | [0.11829608800995549, 0.1423266815096795] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 51.33806553708505 | 49.836290286557 | 8.389552201677043 | [41.67096711047247, 64.8371803393668] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2924.8 | 2922.0 | 23.509572518444482 | [2889.0, 2948.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 48.74650616934163 | 48.7 | 0.39213175730179295 | [48.14919751337478, 49.13333333333333] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 5.133316389171292 | 4.983333333333333 | 0.8389821263558999 | [4.166666666666667, 6.483333333333333] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.27486230125802574 | 0.2754750669557454 | 0.01710542658015274 | [0.25108757176356433, 0.29920180722891565] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.342485660869565 | 8.348711555555553 | 0.016672953541070616 | [8.322335434782607, 8.363340869565219] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1960.8 | 1967.0 | 19.279522815671555 | [1940.0, 1982.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 59.65150000000001 | 59.42 | 0.5991777699481172 | [59.024, 60.555] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 323.0 | 321.0 | 86.92525524840292 | [227.0, 445.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 323.0 | 321.0 | 86.92525524840292 | [227.0, 445.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.015374361927089885 | 0.015556240098574196 | 0.0028499953009731988 | [0.012418113240840814, 0.01966346463799884] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[1].recovery_ms | 5 | 0 | 1875.0 | 1883.0 | 79.94685734911661 | [1784.0, 1956.0] | 0.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[3].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[3].recovery_ms | 5 | 0 | 1823.2 | 1831.0 | 28.2789674493253 | [1789.0, 1858.0] | 0.0 |
| adaptive | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[5].recovery_ms | 5 | 0 | 1892.4 | 1906.0 | 57.851534119675684 | [1798.0, 1948.0] | 0.0 |
| adaptive | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 8000858.88 | 8001630.933333334 | 1232.0209291153137 | [7999349.866666666, 8001981.866666666] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.18918381846360105 | 0.18363879741729097 | 0.020370010668262585 | [0.17052169619579663, 0.22318783080149052] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 54.55975831197965 | 54.00201967553586 | 11.751538707073818 | [39.336530082011336, 72.31541839368327] | 0.0 |
| classic | switch_count | 5 | 0 | 39205.8 | 40062.0 | 1889.2837796371407 | [36802.0, 41002.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 653.4277743704272 | 667.6888718521358 | 31.48680257732173 | [613.3666666666667, 683.3666666666667] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 5.456649222512959 | 5.4 | 1.1758962383629783 | [3.933333333333333, 7.233333333333333] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.3466493295948073 | 0.34977075757793524 | 0.023166135090090946 | [0.30761823239888725, 0.3670310291256701] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.313156 | 8.311244444444446 | 0.004770862450178532 | [8.309876000000001, 8.321418000000001] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1980.6 | 1980.0 | 3.1304951684997055 | [1978.0, 1986.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 53.2124 | 53.125 | 1.7842591739991134 | [51.539, 55.988] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 151.8 | 149.0 | 7.596051605933177 | [146.0, 165.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 151.8 | 149.0 | 7.596051605933177 | [146.0, 165.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.008898518693084279 | 0.008603720168977948 | 0.0005313095830165568 | [0.008389076539285162, 0.009658952766173772] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[1].recovery_ms | 5 | 0 | 1849.8 | 1911.0 | 120.41885234463912 | [1662.0, 1953.0] | 0.0 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[3].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[3].recovery_ms | 5 | 0 | 1853.2 | 1822.0 | 74.95798823340978 | [1778.0, 1949.0] | 0.0 |
| classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[5].recovery_ms | 5 | 0 | 1838.2 | 1798.0 | 65.9787844689488 | [1792.0, 1945.0] | 0.0 |
| classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 7488882.24 | 7520325.866666666 | 53908.3176192403 | [7404517.866666666, 7530853.866666666] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.06418503381623863 | 0.06008874510918272 | 0.007008620399407103 | [0.058558163980378414, 0.07471694788316666] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.2670096290627905 | 0.2768338917921826 | 0.02265374669711473 | [0.2342219737124757, 0.2918858789428965] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 74.02621608069624 | 71.26371826576616 | 30.8534936048813 | [31.90011323122414, 118.84636053908997] | 0.0 |
| edpf | switch_count | 5 | 0 | 6539.8 | 6728.0 | 519.8809479101923 | [5730.0, 7030.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 108.99666666666667 | 112.13333333333334 | 8.664682465169877 | [95.5, 117.16666666666669] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 6.916666666666667 | 6.7 | 2.838671441987528 | [3.0, 11.0] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.42809371353383474 | 0.42949965126096695 | 0.0037931396977947352 | [0.4218788613389506, 0.4312666813585054] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.437894701384275 | 8.324950627906976 | 0.25273641708962574 | [8.27174946511628, 8.879393357142856] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1506.6 | 1465.0 | 74.25833286574645 | [1432.0, 1599.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 46.5289 | 46.463 | 0.3868834449805282 | [46.126, 47.1495] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 2706.6 | 2100.0 | 1515.8717953705716 | [1826.0, 5409.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 2706.6 | 2100.0 | 1515.8717953705716 | [1826.0, 5409.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 2920.0 | 2749.0 | 305.8831476234021 | [2674.0, 3392.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.14677364967573298 | 0.13112177900243258 | 0.03827198570269169 | [0.12019168501489444, 0.21334500376575] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | 6945.0 | None | [4988.0, +inf] | 0.2 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | 4760.0 | None | [2954.0, +inf] | 0.2 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].failover_ms | 5 | 0 | +inf | 4471.0 | None | [3950.0, +inf] | 0.2 |
| edpf | episode[3].recovery_ms | 5 | 0 | +inf | 2794.0 | None | [1828.0, +inf] | 0.2 |
| edpf | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].failover_ms | 5 | 0 | 5379.0 | 4989.0 | 535.6570731354157 | [4987.0, 5989.0] | 0.0 |
| edpf | episode[5].recovery_ms | 5 | 0 | 3030.8 | 2826.0 | 513.8226347680685 | [2593.0, 3792.0] | 0.0 |
| edpf | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 7999700.8 | 8000227.2 | 1789.4159146367017 | [7996542.4, 8000929.066666666] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.1273288672875852 | 0.12119865800898526 | 0.012897182733003174 | [0.11482090117778504, 0.14199364401577982] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 48.13214572521373 | 50.66522777419788 | 17.542974649552665 | [19.67517027525963, 67.99210385033952] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2935.8 | 2953.0 | 25.781776509775273 | [2902.0, 2957.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 48.92983572496014 | 49.21666666666667 | 0.4295275476634633 | [48.36666666666667, 49.28251195813403] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 4.813315944734255 | 5.066666666666666 | 1.7548849981969288 | [1.9666666666666663, 6.8] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.2710919345266797 | 0.27330545809903717 | 0.003914721381016559 | [0.2651799338758901, 0.27442618237577265] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.324734807729468 | 8.322043695652173 | 0.01456160804192018 | [8.308179782608697, 8.343738] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1972.0 | 1978.0 | 11.726039399558575 | [1952.0, 1980.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 59.1865 | 59.064 | 0.3500632085781096 | [58.78, 59.682500000000005] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 231.2 | 207.0 | 78.52197144748723 | [160.0, 340.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 231.2 | 207.0 | 78.52197144748723 | [160.0, 340.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.012717908316267273 | 0.01185719530895661 | 0.002254785936290732 | [0.010491250081117912, 0.01613613349183287] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[1].recovery_ms | 5 | 0 | 1844.6 | 1790.0 | 87.39736838143354 | [1767.0, 1950.0] | 0.0 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[3].recovery_ms | 5 | 0 | 1864.6 | 1889.0 | 74.18423013012941 | [1776.0, 1958.0] | 0.0 |
| enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[5].recovery_ms | 5 | 0 | 1864.0 | 1891.0 | 70.57619995437555 | [1767.0, 1947.0] | 0.0 |
| enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 7695897.813333334 | 7699477.333333333 | 283378.7372879172 | [7377145.066666666, 8000578.133333334] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.03784508687193496 | 0.03731082145044178 | 0.03534815334932839 | [0.0, 0.07827728509965756] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.2860516120685526 | 0.3093253983215811 | 0.08484698662254087 | [0.13805966514025195, 0.3524499129692651] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 56.15240105456911 | 48.329840696845636 | 23.41627434875521 | [27.291497118993888, 89.492217577775] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 1268.2 | 835.0 | 966.5677420646729 | [821.0, 2997.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 21.136666666666667 | 13.916666666666666 | 16.10946236774455 | [13.683333333333334, 49.95] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 5.393333333333333 | 4.833333333333333 | 2.166365363665347 | [2.5166666666666666, 8.333333333333334] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.3775840713076001 | 0.3979265764117575 | 0.07105601559929547 | [0.254474733823384, 0.4294365489428045] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.509312995453204 | 8.31185088888889 | 0.43147050524920355 | [8.122926404761905, 9.079897999999998] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1658.6 | 1699.0 | 246.13878199097354 | [1292.0, 1977.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 50.0685 | 48.5155 | 6.085666972485431 | [45.971, 60.684] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 2321.2 | 2064.0 | 1703.7410894851364 | [160.0, 4467.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 2321.2 | 2064.0 | 1703.7410894851364 | [160.0, 4467.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 1733.0 | 1706.0 | 1619.250752663095 | [0.0, 3566.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.11134507377180203 | 0.11557648475465532 | 0.06414461308170931 | [0.0090639784233099, 0.17776112146503523] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | 3183.2 | 4953.0 | 2934.9236276264496 | [0.0, 5978.0] | 0.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | 2715.4 | 2800.0 | 828.1783020581981 | [1943.0, 3938.0] | 0.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | 3976.4 | 5948.0 | 3653.7274528897196 | [0.0, 6987.0] | 0.0 |
| rtt-threshold | episode[3].recovery_ms | 5 | 0 | 3458.0 | 3800.0 | 1529.4057669565655 | [1772.0, 4954.0] | 0.0 |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].failover_ms | 5 | 0 | 2187.6 | 0.0 | 3017.7198014394908 | [0.0, 5986.0] | 0.0 |
| rtt-threshold | episode[5].recovery_ms | 5 | 0 | 2491.2 | 1950.0 | 835.2156607727132 | [1937.0, 3837.0] | 0.0 |
| rtt-threshold | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[2] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[3] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[4] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[5] graded=True | — | 1.0 | 0.0 |
| adaptive | episode[6] graded=True | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 1.0 | 0.0 |
| classic | episode[2] graded=True | — | 0.0 | 1.0 |
| classic | episode[3] graded=True | — | 1.0 | 0.0 |
| classic | episode[4] graded=True | — | 0.0 | 1.0 |
| classic | episode[5] graded=True | — | 1.0 | 0.0 |
| classic | episode[6] graded=True | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 0.8 | 0.19999999999999996 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | episode[3] graded=True | — | 0.8 | 0.19999999999999996 |
| edpf | episode[4] graded=True | — | 0.0 | 1.0 |
| edpf | episode[5] graded=True | — | 1.0 | 0.0 |
| edpf | episode[6] graded=True | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 0.8 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[4] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[6] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[3] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[4] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[5] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | episode[6] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 0.8 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [0.9996710670584623, 0.999978064884073] | 0.0 | 1.0015424164524422 | () |
| adaptive | edpf | 5 | [1.0622335096344278, 1.0803573544396787] | -6.008874510918272 | 0.6248690185120503 | () |
| adaptive | enhanced | 5 | [0.999824553708496, 1.000307199438264] | 0.0 | 1.0079323109465892 | () |
| adaptive | rtt-threshold | 5 | [0.999868409509606, 1.0842946507147444] | -3.7310821450441782 | 0.6460653970535394 | () |
| classic | adaptive | 5 | [1.0000219355970867, 1.0003290411740189] | 0.0 | 0.9984599589322382 | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| classic | edpf | 5 | [1.0625131060835529, 1.080689115855826] | -6.008874510918272 | 0.6341118188251946 | () |
| classic | enhanced | 5 | [0.9998903364477781, 1.0003949707063393] | 0.0 | 0.9989727786337956 | () |
| classic | rtt-threshold | 5 | [1.0001315904903938, 1.0843897914040388] | -3.7310821450441782 | 0.6683435141933166 | () |
| edpf | adaptive | 5 | [0.9256196534327703, 0.9414125904803685] | 6.008874510918272 | 1.6003353828954723 | () |
| edpf | classic | 5 | [0.9253354968862381, 0.9411648611902986] | 6.008874510918272 | 1.5770089285714286 | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| edpf | enhanced | 5 | [0.9254572568972323, 0.9412887095359241] | 6.008874510918272 | 1.5313634007257646 | () |
| edpf | rtt-threshold | 5 | [0.9386859059218073, 1.019408700616036] | 2.2777923658740935 | 1.438722966014418 | () |
| enhanced | adaptive | 5 | [0.9996928949042491, 1.0001754770783067] | 0.0 | 0.9921301154249738 | () |
| enhanced | classic | 5 | [0.9996051852339278, 1.0001096755796355] | 0.0 | 1.0010282776349615 | () |
| enhanced | edpf | 5 | [1.0623733078589903, 1.080546932391763] | -6.008874510918272 | 0.6530128639133378 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [1.0, 1.0839616583022145] | -3.7310821450441782 | 0.6612410986775178 | () |
| rtt-threshold | adaptive | 5 | [0.9222585386184657, 1.0001316078087301] | 3.7310821450441782 | 1.5478309232480534 | () |
| rtt-threshold | classic | 5 | [0.9221776227764251, 0.999868426823385] | 3.7310821450441782 | 1.496236559139785 | () |
| rtt-threshold | edpf | 5 | [0.9809608250309153, 1.06531907392173] | -2.2777923658740935 | 0.6950608446671439 | () |
| rtt-threshold | enhanced | 5 | [0.9225418559234634, 1.0] | 3.7310821450441782 | 1.5123076923076924 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M4@baseline--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| upstream-classic | useful_goodput_bps | 5 | 0 | 8000051.733333332 | 8000753.6 | 1436.256566371041 | [7997770.666666667, 8001280.0] | 0.0 |
| upstream-classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-classic | per_link_share_gini | 5 | 0 | 0.05405563572786558 | 0.043643803878970676 | 0.017116770687871513 | [0.03857024740436499, 0.07682982646632956] | 0.0 |
| upstream-classic | cpu_ms_per_mb | 5 | 0 | 57.33385025403358 | 58.336794649815886 | 8.954346310418451 | [47.99442624729848, 66.68524962289491] | 0.0 |
| upstream-classic | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | sender_cpu_percent | 5 | 0 | 5.733291667361099 | 5.833236112731455 | 0.8947060113897777 | [4.8, 6.666555557407377] | 0.0 |
| upstream-classic | diagnostics.loss_ratio | 5 | 0 | 0.43392089427950414 | 0.43346341585835646 | 0.0023615789655578113 | [0.4319106201355762, 0.43787919397675495] | 0.0 |
| upstream-classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.300799289855073 | 8.294335 | 0.010441806505339116 | [8.291675652173911, 8.315019999999999] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1967.2 | 1975.0 | 19.62651268055535 | [1933.0, 1981.0] | 0.0 |
| upstream-classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-classic | diagnostics.ms_rtt_median | 5 | 0 | 53.04 | 53.202 | 0.5676346756497531 | [52.1155, 53.5365] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_delta | 5 | 0 | 169.8 | 136.0 | 53.08201201913884 | [125.0, 242.0] | 0.0 |
| upstream-classic | diagnostics.pkt_belated_sum | 5 | 0 | 169.8 | 136.0 | 53.08201201913884 | [125.0, 242.0] | 0.0 |
| upstream-classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-classic | diagnostics.retrans_ratio | 5 | 0 | 0.007590672134276696 | 0.006587501083470573 | 0.0015584436791702496 | [0.00637887882345124, 0.009789189905864675] | 0.0 |
| upstream-classic | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-classic | episode[1].recovery_ms | 5 | 0 | 1824.0 | 1805.0 | 71.82965961216857 | [1777.0, 1950.0] | 0.0 |
| upstream-classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[3].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-classic | episode[3].recovery_ms | 5 | 0 | 1824.4 | 1793.0 | 70.71279940717947 | [1778.0, 1949.0] | 0.0 |
| upstream-classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[5].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-classic | episode[5].recovery_ms | 5 | 0 | 1858.2 | 1804.0 | 84.93055987099108 | [1788.0, 1954.0] | 0.0 |
| upstream-classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| upstream-enhanced | useful_goodput_bps | 5 | 0 | 7998788.373333333 | 7998648.0 | 872.0534458646779 | [7997770.666666667, 8000051.733333333] | 0.0 |
| upstream-enhanced | viewer_loss_ratio | 5 | 0 | 8.700191404210892e-06 | 0.0 | 1.9454219397074906e-05 | [0.0, 4.3500957021054466e-05] | 0.0 |
| upstream-enhanced | per_link_share_gini | 5 | 0 | 0.20361109812818728 | 0.20513325212958625 | 0.033720401492242376 | [0.148170560009662, 0.23548580382562542] | 0.0 |
| upstream-enhanced | cpu_ms_per_mb | 5 | 0 | 57.90939819267402 | 63.517700265807406 | 8.943121545420144 | [47.17153476905483, 65.3443765329674] | 0.0 |
| upstream-enhanced | switch_count | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | switches_per_second | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | sender_cpu_percent | 5 | 0 | 5.78998427803981 | 6.35 | 0.8937206682964126 | [4.716588056865719, 6.533333333333333] | 0.0 |
| upstream-enhanced | diagnostics.loss_ratio | 5 | 0 | 0.33201892833922125 | 0.3288183227339234 | 0.017818099328901684 | [0.31596692015783984, 0.3621641566676688] | 0.0 |
| upstream-enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.366884547826086 | 8.324004999999998 | 0.11740504783591807 | [8.294538913043478, 8.575614888888888] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1966.8 | 1973.0 | 15.401298646542763 | [1940.0, 1979.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| upstream-enhanced | diagnostics.ms_rtt_median | 5 | 0 | 61.9859 | 61.9045 | 0.6785036293196982 | [61.262, 62.7655] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 486.4 | 287.0 | 579.232941052216 | [114.0, 1513.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 486.4 | 287.0 | 579.232941052216 | [114.0, 1513.0] | 0.0 |
| upstream-enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.4 | 0.0 | 0.894427190999916 | [0.0, 2.0] | 0.0 |
| upstream-enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| upstream-enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.0167027971250393 | 0.012665875498975078 | 0.014004458164082537 | [0.006912242686890574, 0.0413515308636685] | 0.0 |
| upstream-enhanced | episode[1].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-enhanced | episode[1].recovery_ms | 5 | 0 | 1790.6 | 1788.0 | 11.802542099056458 | [1777.0, 1806.0] | 0.0 |
| upstream-enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[3].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-enhanced | episode[3].recovery_ms | 5 | 0 | 1790.0 | 1789.0 | 11.269427669584644 | [1779.0, 1802.0] | 0.0 |
| upstream-enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[5].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| upstream-enhanced | episode[5].recovery_ms | 5 | 0 | 1843.0 | 1813.0 | 66.32118816788493 | [1793.0, 1950.0] | 0.0 |
| upstream-enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| upstream-enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| upstream-classic | episode[1] graded=True | — | 1.0 | 0.0 |
| upstream-classic | episode[2] graded=True | — | 0.0 | 1.0 |
| upstream-classic | episode[3] graded=True | — | 1.0 | 0.0 |
| upstream-classic | episode[4] graded=True | — | 0.0 | 1.0 |
| upstream-classic | episode[5] graded=True | — | 1.0 | 0.0 |
| upstream-classic | episode[6] graded=True | — | 0.0 | 1.0 |
| upstream-classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-classic | settled_rate | — | 1.0 | — |
| upstream-classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| upstream-enhanced | episode[1] graded=True | — | 1.0 | 0.0 |
| upstream-enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| upstream-enhanced | episode[3] graded=True | — | 1.0 | 0.0 |
| upstream-enhanced | episode[4] graded=True | — | 0.0 | 1.0 |
| upstream-enhanced | episode[5] graded=True | — | 1.0 | 0.0 |
| upstream-enhanced | episode[6] graded=True | — | 0.0 | 1.0 |
| upstream-enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| upstream-enhanced | settled_rate | — | 1.0 | — |
| upstream-enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| upstream-enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| upstream-classic | upstream-classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| upstream-classic | upstream-enhanced | 5 | [0.999890314796534, 1.0003729706011408] | 0.0 | 1.0011142061281337 | () |
| upstream-enhanced | upstream-classic | 5 | [0.9996271684540651, 1.0001096972356296] | 0.0 | 0.9988870339454646 | () |
| upstream-enhanced | upstream-enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M4@fec-pair--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| enhanced | useful_goodput_bps | 3 | 0 | 8000636.622222222 | 8000578.133333334 | 268.0297606468765 | [8000402.666666667, 8000929.066666666] | 0.0 |
| enhanced | viewer_loss_ratio | 3 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 3 | 0 | 0.1306393832430305 | 0.12813485152530532 | 0.008563886660588424 | [0.12360698583827247, 0.14017631236551378] | 0.0 |
| enhanced | cpu_ms_per_mb | 3 | 0 | 64.93926052563357 | 64.49675366339895 | 0.7676697098407159 | [64.49533913682504, 65.82568877667673] | 0.0 |
| enhanced | switch_count | 3 | 0 | 2904.6666666666665 | 2901.0 | 15.821925715074423 | [2891.0, 2922.0] | 0.0 |
| enhanced | switches_per_second | 3 | 0 | 48.41030427270656 | 48.349194180097 | 0.26369436701179144 | [48.18253029116181, 48.699188346860886] | 0.0 |
| enhanced | sender_cpu_percent | 3 | 0 | 6.494336205507686 | 6.449892501791637 | 0.07697875291273497 | [6.449892501791637, 6.5832236129397845] | 0.0 |
| enhanced | diagnostics.loss_ratio | 3 | 0 | 0.27402504255085874 | 0.2715488797410678 | 0.009116024089411591 | [0.2664029118531413, 0.28412333605836715] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 3 | 0 | 8.335799130434783 | 8.332541956521741 | 0.005959335550081783 | [8.332178260869567, 8.342677173913042] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 3 | 0 | 1973.0 | 1980.0 | 14.798648586948742 | [1956.0, 1983.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 3 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 3 | 0 | 60.037499999999994 | 60.2135 | 0.9039430291782798 | [59.058499999999995, 60.8405] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 3 | 0 | 284.6666666666667 | 266.0 | 38.552993831002716 | [259.0, 329.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 3 | 0 | 284.6666666666667 | 266.0 | 38.552993831002716 | [259.0, 329.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 3 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 3 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 3 | 0 | 0.014525381364343419 | 0.01397666242477838 | 0.0014002305363678277 | [0.0134826128225041, 0.016116868845747777] | 0.0 |
| enhanced | episode[1].failover_ms | 3 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[1].recovery_ms | 3 | 0 | 1796.0 | 1796.0 | 1.0 | [1795.0, 1797.0] | 0.0 |
| enhanced | episode[2].failover_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 3 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[3].recovery_ms | 3 | 0 | 1801.6666666666667 | 1800.0 | 4.725815626252609 | [1798.0, 1807.0] | 0.0 |
| enhanced | episode[4].failover_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 3 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[5].recovery_ms | 3 | 0 | 1795.0 | 1797.0 | 4.358898943540674 | [1790.0, 1798.0] | 0.0 |
| enhanced | episode[6].failover_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 3 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| enhanced | episode[1] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[4] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[6] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| enhanced | enhanced | 3 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M4@fec-pair--production--slt:4001--fec:on — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| enhanced | useful_goodput_bps | 3 | 0 | 7675145.955555555 | 7614902.4 | 123190.1386464088 | [7593670.933333334, 7816864.533333333] | 0.0 |
| enhanced | viewer_loss_ratio | 3 | 0 | 0.04067969862789125 | 0.04803522435613771 | 0.015280722554503036 | [0.023112275482338313, 0.05089159604519774] | 0.0 |
| enhanced | per_link_share_gini | 3 | 0 | 0.3415513967701503 | 0.33405905747249554 | 0.01936592088395358 | [0.3270509905601991, 0.36354414227775633] | 0.0 |
| enhanced | cpu_ms_per_mb | 3 | 0 | 104.33710512241974 | 106.75556970847232 | 4.387518266714127 | [99.2725403761208, 106.98320528266609] | 0.0 |
| enhanced | switch_count | 3 | 0 | 1265.6666666666667 | 1202.0 | 121.70592973776311 | [1189.0, 1406.0] | 0.0 |
| enhanced | switches_per_second | 3 | 0 | 21.0943343536867 | 20.033333333333335 | 2.0285361934085206 | [19.816336394393428, 23.433333333333337] | 0.0 |
| enhanced | sender_cpu_percent | 3 | 0 | 10.005498982424367 | 10.133333333333333 | 0.26574044745403763 | [9.7, 10.183163613939769] | 0.0 |
| enhanced | diagnostics.loss_ratio | 3 | 0 | 0.35527654921110113 | 0.358098529609586 | 0.006926557604756999 | [0.3473844695799081, 0.3603466484438094] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 3 | 0 | 11.560571485200846 | 11.469890232558138 | 0.2004688351650785 | [11.42146581395349, 11.790358409090912] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 3 | 0 | 1615.6666666666667 | 1653.0 | 69.92376801441219 | [1535.0, 1659.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 3 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 3 | 0 | 44.173 | 44.13 | 0.15989684174491986 | [44.039, 44.35] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 3 | 0 | 5011.0 | 4787.0 | 422.2049265463396 | [4748.0, 5498.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 3 | 0 | 5011.0 | 4787.0 | 422.2049265463396 | [4748.0, 5498.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 3 | 0 | 1839.3333333333333 | 2171.0 | 694.6641874555887 | [1041.0, 2306.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 3 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 3 | 0 | 0.13661722933783577 | 0.1372150756037012 | 0.0074217519674186746 | [0.12891463568632405, 0.14372197672348208] | 0.0 |
| enhanced | episode[1].failover_ms | 3 | 0 | +inf | 0.0 | None | [0.0, +inf] | 0.3333333333333333 |
| enhanced | episode[1].recovery_ms | 3 | 0 | +inf | 1811.0 | None | [1800.0, +inf] | 0.3333333333333333 |
| enhanced | episode[2].failover_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[2].recovery_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[3].failover_ms | 3 | 0 | 6618.666666666667 | 6948.0 | 575.6260360106493 | [5954.0, 6954.0] | 0.0 |
| enhanced | episode[3].recovery_ms | 3 | 0 | 4260.333333333333 | 4174.0 | 508.03182315021695 | [3801.0, 4806.0] | 0.0 |
| enhanced | episode[4].failover_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 3 | 0 | 3302.0 | 3955.0 | 3028.763278963874 | [0.0, 5951.0] | 0.0 |
| enhanced | episode[5].recovery_ms | 3 | 0 | 2466.0 | 1797.0 | 1162.2078127426264 | [1793.0, 3808.0] | 0.0 |
| enhanced | episode[6].failover_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 3 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 3 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| enhanced | episode[1] graded=True | — | 0.6666666666666666 | 0.33333333333333337 |
| enhanced | episode[2] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[3] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[4] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=True | — | 1.0 | 0.0 |
| enhanced | episode[6] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| enhanced | enhanced | 3 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M5@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 7311169.6 | 7308888.533333333 | 16789.87712800017 | [7292745.6, 7333278.4] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.29146646556162975 | 0.2911336642889819 | 0.0012634857127572477 | [0.2898833675704124, 0.29297375699925793] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.18743053905612314 | 0.1882138677090113 | 0.024140707481634004 | [0.15458056277881624, 0.21582203955001497] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 136.01543583609362 | 136.20841694153344 | 4.377449602646523 | [128.91005656260552, 139.84151877212156] | 0.0 |
| adaptive | switch_count | 5 | 0 | 7.6 | 8.0 | 0.5477225575051662 | [7.0, 8.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 0.12666627778425915 | 0.13333333333333333 | 0.009129241816803229 | [0.1166647222546291, 0.13333333333333333] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 12.429958611800917 | 12.416459725671238 | 0.3861149071259553 | [11.816666666666666, 12.8] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.4809133409765066 | 0.4801924656187469 | 0.001458093356665484 | [0.479681458826245, 0.4832334433848313] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.12579919047619 | 8.131703333333334 | 0.01825241393316872 | [8.096442380952382, 8.144540952380952] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1412.2 | 1412.0 | 3.96232255123179 | [1407.0, 1417.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 556.8770999999999 | 562.1605 | 17.59160813214639 | [526.326, 570.358] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 389.6 | 361.0 | 107.50255810909803 | [285.0, 537.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 389.6 | 361.0 | 107.50255810909803 | [285.0, 537.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 17216.2 | 17183.0 | 118.47657996414313 | [17075.0, 17371.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.18592023520628337 | 0.1853212234539869 | 0.003060104845866251 | [0.18172715337887452, 0.19007164563139095] | 0.0 |
| adaptive | episode[2].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | episode[2].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| adaptive | load[1].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 7319697.280000001 | 7344859.2 | 66450.06298919175 | [7226419.2, 7385918.4] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.2913742991497009 | 0.29168040253318295 | 0.006261486925846129 | [0.28493980024627175, 0.2997600319957339] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.20655159620530794 | 0.202197820574164 | 0.008007079632653371 | [0.20017220913563527, 0.21750067611241727] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 136.73713247143584 | 137.04387561963023 | 1.982279756631555 | [133.9712543434461, 138.7161096570454] | 0.0 |
| classic | switch_count | 5 | 0 | 8.2 | 8.0 | 0.4472135954999579 | [8.0, 9.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 0.13666616667499984 | 0.13333333333333333 | 0.0074524419096441445 | [0.13333333333333333, 0.14999750004166598] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 12.50995794514536 | 12.516666666666667 | 0.1293784920887145 | [12.3, 12.616666666666667] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.4797587255934238 | 0.47767205184373857 | 0.0037522236917728507 | [0.4771221315477436, 0.4858224840936622] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.18689103986711 | 8.17947857142857 | 0.026135441979858214 | [8.158482380952382, 8.228011627906978] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1410.8 | 1410.0 | 3.7682887362833544 | [1407.0, 1417.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 516.6684 | 524.499 | 73.42288464385611 | [396.8135, 576.559] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 444.4 | 309.0 | 212.88565005655033 | [266.0, 714.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 444.4 | 309.0 | 212.88565005655033 | [266.0, 714.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 17220.2 | 17122.0 | 540.5068917229456 | [16661.0, 17988.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.19017085255097105 | 0.18937987374675083 | 0.0027764288201232532 | [0.1875571820677036, 0.19413597858052056] | 0.0 |
| classic | episode[2].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | episode[2].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | load[1].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 7282708.906666666 | 7284849.6 | 55507.73669545949 | [7216944.0, 7366266.133333334] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.2956745578617238 | 0.2936222638516193 | 0.005479879429915646 | [0.2889916229274566, 0.3015401467122515] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.1765028433989951 | 0.182594090279655 | 0.044339946571557334 | [0.12149316707964045, 0.2227045591579213] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 140.2769987549565 | 140.2255580755511 | 2.056793314945686 | [137.69843373822678, 143.3111261487128] | 0.0 |
| edpf | switch_count | 5 | 0 | 7.4 | 7.0 | 0.5477225575051661 | [7.0, 8.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 0.12333216668611077 | 0.1166647222546291 | 0.00912977429008683 | [0.1166647222546291, 0.13333333333333333] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 12.769872724343482 | 12.833333333333334 | 0.21392062688522626 | [12.483125281245313, 13.04978250362494] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.48492179212332054 | 0.48454362538869583 | 0.004650235233337809 | [0.4786198355636589, 0.49163386959423694] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.173060242740998 | 8.150683414634148 | 0.07808496040998045 | [8.098547857142858, 8.298197560975607] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1416.2 | 1417.0 | 2.588435821108957 | [1413.0, 1419.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 529.3731 | 558.053 | 76.47232077555907 | [394.032, 581.296] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 494.2 | 570.0 | 176.8380615139173 | [264.0, 685.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 494.2 | 570.0 | 176.8380615139173 | [264.0, 685.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 17448.6 | 17435.0 | 552.7506671185482 | [16628.0, 18169.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.188490004300021 | 0.1867077145961608 | 0.005578648114978321 | [0.18204400283889283, 0.19546156641831727] | 0.0 |
| edpf | episode[2].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| edpf | episode[2].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | load[1].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 7318609.386666666 | 7310643.2 | 55518.71810188413 | [7255897.6, 7397850.133333334] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.2913956172815776 | 0.2896942051022132 | 0.004948419217860914 | [0.2852063301282051, 0.2981140306721702] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.2022700203194116 | 0.2155049715352466 | 0.028870158702152055 | [0.15258118993914271, 0.22451496325519107] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 139.23132400098623 | 139.70498975156295 | 2.406024211683907 | [135.21937192289715, 141.70824418285827] | 0.0 |
| enhanced | switch_count | 5 | 0 | 8.2 | 8.0 | 0.4472135954999579 | [8.0, 9.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 0.1366648333638884 | 0.13333111114814752 | 0.007453187315328287 | [0.13333111114814752, 0.14999750004166598] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 12.736496947273102 | 12.766666666666667 | 0.19875080875585016 | [12.416459725671238, 12.899785003583274] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.4810204396982371 | 0.4810636490922029 | 0.002390658011599675 | [0.4786199746572975, 0.4840098954233667] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.184690242524917 | 8.197481666666667 | 0.07084733755462704 | [8.089487142857143, 8.281919069767444] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1415.8 | 1414.0 | 8.555699854482976 | [1404.0, 1426.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 537.957 | 560.858 | 41.646512382491274 | [484.898, 578.8605] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 437.6 | 391.0 | 117.918616002733 | [323.0, 594.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 437.6 | 391.0 | 117.918616002733 | [323.0, 594.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 17307.2 | 17147.0 | 312.7094498092439 | [17043.0, 17767.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.19141571492717288 | 0.1927391573182581 | 0.005968925533954634 | [0.18323366555924697, 0.19949005164861508] | 0.0 |
| enhanced | episode[2].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | episode[2].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | load[1].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 7264425.279999999 | 7257827.733333333 | 51169.912924564676 | [7202906.666666667, 7340121.6] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.29617930119806646 | 0.2958653298011468 | 0.005996333333052348 | [0.287673797246473, 0.3039648310658918] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.21289026199936828 | 0.2066312296342119 | 0.014012109578009002 | [0.20265188596250786, 0.23735771461710098] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 138.87699122088753 | 139.60295740209136 | 1.646042496912883 | [136.60082506898343, 140.31372519982673] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 7.2 | 7.0 | 0.8366600265340756 | [6.0, 8.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 0.11999888890740709 | 0.1166647222546291 | 0.013945163794248835 | [0.09999833336111064, 0.13333333333333333] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 12.609873390999038 | 12.633122781286978 | 0.07771327118120187 | [12.533333333333331, 12.71645472575457] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.4825328370027425 | 0.4837886899974724 | 0.004083540085670208 | [0.47814283403803054, 0.48748195879217665] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 8.102610095238095 | 8.066316428571428 | 0.09020434774696118 | [8.001136190476192, 8.216260714285715] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1417.8 | 1419.0 | 6.300793600809346 | [1411.0, 1426.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 516.2751000000001 | 563.4159999999999 | 71.53705451582417 | [416.0945, 572.478] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 489.4 | 494.0 | 149.71406079590523 | [271.0, 678.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 489.4 | 494.0 | 149.71406079590523 | [271.0, 678.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 17631.6 | 17646.0 | 506.68362910202654 | [16904.0, 18254.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.1837563616876645 | 0.1825176841031554 | 0.0061245474935010995 | [0.17714222123302917, 0.19141965333274305] | 0.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | episode[2].recovery_ms | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | load[1].reached_ms | 5 | 0 | 3000.0 | 3000.0 | 0.0 | [3000.0, 3000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[2] graded=True | — | 1.0 | 0.0 |
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | load[1] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[2] graded=True | — | 1.0 | 0.0 |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | load[1] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[2] graded=True | — | 1.0 | 0.0 |
| edpf | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| edpf | load[1] graded=True | 1.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[2] graded=True | — | 1.0 | 0.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | load[1] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[2] graded=True | — | 1.0 | 0.0 |
| rtt-threshold | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | load[1] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| adaptive | classic | 5 | [0.9909467765759756, 1.0147872960372961] | -0.054673824420103934 | 1.0 | () |
| adaptive | edpf | 5 | [0.9955217836640385, 1.0146365183564308] | -0.2488599562637417 | 1.0 | () |
| adaptive | enhanced | 5 | [0.9857925571025354, 1.0091893983362354] | 0.1439459186768688 | 1.0 | () |
| adaptive | rtt-threshold | 5 | [0.9943105756358769, 1.0124725943970767] | -0.47316655121649176 | 1.0 | () |
| classic | adaptive | 5 | [0.9854281817529251, 1.0091359330672693] | 0.054673824420103934 | 1.0 | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| classic | edpf | 5 | [0.9810152211714823, 1.0234135667396063] | -0.19418613184363775 | 1.0 | () |
| classic | enhanced | 5 | [0.9904934791955285, 1.0179193267556588] | 0.19861974309697272 | 1.0 | () |
| classic | rtt-threshold | 5 | [0.9924094556495338, 1.0202137715407549] | -0.4184927267963878 | 1.0 | () |
| edpf | adaptive | 5 | [0.985574618997412, 1.0044983609695404] | 0.2488599562637417 | 1.0 | () |
| edpf | classic | 5 | [0.9771220868077827, 1.0193521756021757] | 0.19418613184363775 | 1.0 | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| edpf | enhanced | 5 | [0.9858162757050354, 1.0114928681572861] | 0.3928058749406105 | 1.0 | () |
| edpf | rtt-threshold | 5 | [0.988071332950851, 1.0124969549330085] | -0.22430659495275007 | 1.0 | () |
| enhanced | adaptive | 5 | [0.9908942777724528, 1.0144122034550793] | -0.1439459186768688 | 1.0 | () |
| enhanced | classic | 5 | [0.982396122870786, 1.009597762129835] | -0.19861974309697272 | 1.0 | () |
| enhanced | edpf | 5 | [0.9886377170624806, 1.0143877968385344] | -0.3928058749406105 | 1.0 | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |
| enhanced | rtt-threshold | 5 | [0.995983935742972, 1.0270645554202193] | -0.6171124698933605 | 1.0 | () |
| rtt-threshold | adaptive | 5 | [0.987681054809682, 1.0057219791316054] | 0.47316655121649176 | 1.0 | () |
| rtt-threshold | classic | 5 | [0.9801867293849333, 1.0076486013986015] | 0.4184927267963878 | 1.0 | () |
| rtt-threshold | edpf | 5 | [0.9876572913408561, 1.0120726780054676] | 0.22430659495275007 | 1.0 | () |
| rtt-threshold | enhanced | 5 | [0.9736486326225658, 1.004032258064516] | 0.6171124698933605 | 1.0 | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | 1.0 | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M6@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 12803872.853333334 | 12804387.555555556 | 956.074198473839 | [12802515.91111111, 12804621.51111111] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.032847587579304754 | 0.032201603929407074 | 0.002595992530424502 | [0.030536556754950706, 0.03721429882010119] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 44.486507882651196 | 44.710982378041074 | 0.6675957145296753 | [43.324817599740676, 44.98375836413286] | 0.0 |
| adaptive | switch_count | 5 | 0 | 2249.8 | 2249.0 | 19.690099034794112 | [2224.0, 2277.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 49.99511457770074 | 49.977777777777774 | 0.43804131153832 | [49.42112397502278, 50.6] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 7.119936297711902 | 7.155396546743406 | 0.10723220491632797 | [6.933333333333334, 7.2] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.006551436268349106 | 0.00876774643237096 | 0.00409414836832676 | [0.001959008667689294, 0.009988087601942638] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.237185185185186 | 13.237624074074075 | 0.0032791109446177 | [13.23257962962963, 13.241661111111114] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1989.8 | 1990.0 | 0.4472135954999579 | [1989.0, 1990.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 61.77660000000001 | 61.7645 | 0.05021503758835604 | [61.724999999999994, 61.8595] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 39.6 | 40.0 | 4.277849927241488 | [34.0, 45.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 39.6 | 40.0 | 4.277849927241488 | [34.0, 45.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.002639704445954607 | 0.0026471677156608663 | 0.00031480101602180755 | [0.0022035812825213414, 0.002997779422649889] | 0.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 12802235.164444445 | 12802281.955555556 | 1932.0819792950667 | [12799942.4, 12805089.422222223] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.08103355763139387 | 0.08323075399874691 | 0.007091287945496128 | [0.07200409109264513, 0.09027967313031438] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 49.60264105719195 | 48.60244637497682 | 1.9782287986458122 | [47.90637478048466, 52.07880826354866] | 0.0 |
| classic | switch_count | 5 | 0 | 16496.4 | 14353.0 | 4951.710946733462 | [11626.0, 21867.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 366.58093978158513 | 318.9484678118264 | 110.03300051349449 | [258.35555555555555, 485.9225350547766] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 7.9376696320328675 | 7.777604942112397 | 0.3155252200487062 | [7.666666666666667, 8.333148152263282] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.18579211462183684 | 0.11842534586269904 | 0.1227606191888683 | [0.07687049253170182, 0.3292308457340345] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.235451481481482 | 13.239059259259266 | 0.008200822313982106 | [13.221666666666668, 13.2426537037037] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1991.8 | 1992.0 | 0.8366600265340756 | [1991.0, 1993.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 65.779 | 62.9565 | 4.16642013604005 | [62.7675, 71.9665] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 56.2 | 59.0 | 18.308467986153293 | [39.0, 83.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 56.2 | 59.0 | 18.308467986153293 | [39.0, 83.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.0029473605553969246 | 0.0028121068600606824 | 0.00037426032838073104 | [0.002591824644549763, 0.003521387797464601] | 0.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 12803264.56888889 | 12802983.822222222 | 865.9516539975822 | [12802281.955555556, 12804387.555555556] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.023907297671456013 | 0.016050838034512904 | 0.013892452293409383 | [0.010247146660527645, 0.04168428389422976] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 46.76585402980038 | 46.78951714884398 | 0.30172892069198376 | [46.378077643678175, 47.21380504997748] | 0.0 |
| edpf | switch_count | 5 | 0 | 6155.8 | 5972.0 | 1145.2520246653137 | [5059.0, 8048.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 136.79371767047152 | 132.7111111111111 | 25.449389373309216 | [112.4197240061332, 178.84047021177307] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 7.484344397285 | 7.488722472833937 | 0.04812620240492497 | [7.4222222222222225, 7.555387658052044] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.03577656186250801 | 0.03564162056041406 | 0.0033039113401298528 | [0.030960751955227097, 0.04019538188277087] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.23829111111111 | 13.237785185185183 | 0.001920492424423389 | [13.23661851851852, 13.24151851851852] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1992.8 | 1991.0 | 2.6832815729997477 | [1991.0, 1997.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 63.6928 | 63.631 | 0.1060545850022523 | [63.6085, 63.86] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 36.4 | 35.0 | 7.436396977031283 | [30.0, 49.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 36.4 | 35.0 | 7.436396977031283 | [30.0, 49.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.002609760475901695 | 0.00257250198952492 | 0.00025078267465814636 | [0.0024056699790891765, 0.0030172892525267485] | 0.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 12803264.56888889 | 12804153.6 | 1455.4216971191615 | [12801346.133333333, 12804387.555555556] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.03697949397723645 | 0.02995291125101704 | 0.01269777973312346 | [0.029311951423163733, 0.0590327986836785] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 44.4609203283764 | 44.846558402909366 | 1.763986188922413 | [41.51354785609599, 46.234151960133666] | 0.0 |
| enhanced | switch_count | 5 | 0 | 2228.0 | 2243.0 | 29.008619408720573 | [2179.0, 2248.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 49.5108959060169 | 49.84444444444444 | 0.645090399255411 | [48.42114619675118, 49.95555555555555] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 7.115522667397515 | 7.177777777777778 | 0.28210203215353247 | [6.644444444444445, 7.399835559209795] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.005354642470938131 | 0.003356880683180552 | 0.0035210263133714594 | [0.002418491304508363, 0.009716747639563664] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.238194074074073 | 13.23821481481481 | 0.0011455536695748026 | [13.23668703703704, 13.239607407407409] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1991.8 | 1993.0 | 3.1144823004794877 | [1988.0, 1995.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 61.8158 | 61.821 | 0.07315445987771398 | [61.7285, 61.9165] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 42.0 | 42.0 | 2.5495097567963922 | [39.0, 45.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 42.0 | 42.0 | 2.5495097567963922 | [39.0, 45.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.0027392494358931545 | 0.0027019524382344774 | 8.681144274316789e-05 | [0.0026463349186668394, 0.002832022211938918] | 0.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 12802656.284444444 | 12802048.0 | 1350.0674893417136 | [12801346.133333333, 12804621.51111111] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.03424272866769783 | 0.037466338290328416 | 0.01410554615763914 | [0.01219224421352838, 0.04770636550225335] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 44.43523420462894 | 44.71588498763314 | 1.3140607059616134 | [42.21083936587322, 45.40940116248067] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 2229.6 | 2247.0 | 86.5291858276732 | [2089.0, 2318.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 49.54578757509093 | 49.93333333333333 | 1.922883912001153 | [46.421190640208, 51.50996644519011] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 7.110985385509951 | 7.155396546743406 | 0.21025729695442985 | [6.755405435434768, 7.266666666666667] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.006038450885932153 | 0.0027879839737080187 | 0.00722258223714685 | [0.0021054964539007092, 0.018916610637640604] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 13.23694148148148 | 13.23887037037037 | 0.005266854290299087 | [13.22904074074074, 13.242681481481483] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1985.2 | 1990.0 | 11.861703081766969 | [1964.0, 1991.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 61.8984 | 61.9095 | 0.08720263757478777 | [61.753, 61.9845] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 72.4 | 43.0 | 69.20115605970756 | [37.0, 196.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 72.4 | 43.0 | 69.20115605970756 | [37.0, 196.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 0.0 | 0.0 | 0.0 | [0.0, 0.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.003332442503204461 | 0.002904879086720817 | 0.0013343517189876827 | [0.002498611882287618, 0.005686238192257826] | 0.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9999634589735624, 1.0003472793405348] | 0.0 | None | () |
| adaptive | edpf | 5 | [0.9999086424264572, 1.0001827451983698] | 0.0 | None | () |
| adaptive | enhanced | 5 | [0.9999269126057484, 1.0002010233918128] | 0.0 | None | () |
| adaptive | rtt-threshold | 5 | [0.9998903729148015, 1.0002558619807371] | 0.0 | None | () |
| classic | adaptive | 5 | [0.9996528412205372, 1.000036542361733] | 0.0 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.99974419879408, 1.0000913609121476] | 0.0 | None | () |
| classic | enhanced | 5 | [0.9996528412205372, 1.0002375730994153] | 0.0 | None | () |
| classic | rtt-threshold | 5 | [0.9997259322870038, 1.0001279099513944] | 0.0 | None | () |
| edpf | adaptive | 5 | [0.999817288191336, 1.0000913659205117] | 0.0 | None | () |
| edpf | classic | 5 | [0.9999086474339063, 1.0002558666569195] | 0.0 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.9998355563676229, 1.0001461988304092] | 0.0 | None | () |
| edpf | rtt-threshold | 5 | [0.9999817288191337, 1.0000731034230679] | 0.0 | None | () |
| enhanced | adaptive | 5 | [0.9997990170104695, 1.0000730927364092] | 0.0 | None | () |
| enhanced | classic | 5 | [0.9997624833281566, 1.0003472793405348] | 0.0 | None | () |
| enhanced | edpf | 5 | [0.9998538225405643, 1.0001644706785329] | 0.0 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.9998903628988051, 1.0002375861249704] | 0.0 | None | () |
| rtt-threshold | adaptive | 5 | [0.9997442034678702, 1.0001096391046138] | 0.0 | None | () |
| rtt-threshold | classic | 5 | [0.9998721064074688, 1.0002741428466992] | 0.0 | None | () |
| rtt-threshold | edpf | 5 | [0.9999269019206519, 1.0000182715147086] | 0.0 | None | () |
| rtt-threshold | enhanced | 5 | [0.9997624703087885, 1.000109649122807] | 0.0 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M7@--production--slt:4001--fec:off — m4a-ours-new / ours-new-200 / ours-new / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| adaptive | useful_goodput_bps | 5 | 0 | 9038287.999999998 | 9076715.2 | 124457.45653394092 | [8900546.666666666, 9152867.733333332] | 0.0 |
| adaptive | viewer_loss_ratio | 5 | 0 | 0.24531256923856987 | 0.24144836384133855 | 0.010031330268216706 | [0.23573249072183383, 0.25667208511895967] | 0.0 |
| adaptive | per_link_share_gini | 5 | 0 | 0.028293366723365493 | 0.030925621354605675 | 0.012886941346831675 | [0.011241938673635099, 0.04239870693854764] | 0.0 |
| adaptive | cpu_ms_per_mb | 5 | 0 | 116.60540103538747 | 116.48854349129886 | 3.4905810287626773 | [113.47991656035876, 121.94007559086259] | 0.0 |
| adaptive | switch_count | 5 | 0 | 1104.8 | 1109.0 | 39.35987804859156 | [1047.0, 1155.0] | 0.0 |
| adaptive | switches_per_second | 5 | 0 | 18.413150225274023 | 18.483333333333334 | 0.6560233077244456 | [17.44970917151381, 19.2496791720138] | 0.0 |
| adaptive | sender_cpu_percent | 5 | 0 | 13.169867835536076 | 13.099781670305497 | 0.24189002712650204 | [12.983116948050863, 13.566440559324011] | 0.0 |
| adaptive | diagnostics.loss_ratio | 5 | 0 | 0.4459901543856321 | 0.44683067958807937 | 0.007840947394038832 | [0.4345935026385224, 0.4562744202567616] | 0.0 |
| adaptive | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.651103819758672 | 10.664714230769228 | 0.08133143632821599 | [10.531603333333331, 10.736889230769236] | 0.0 |
| adaptive | diagnostics.ms_rcv_buf_min | 5 | 0 | 1806.6 | 1797.0 | 34.03380672214026 | [1776.0, 1857.0] | 0.0 |
| adaptive | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| adaptive | diagnostics.ms_rtt_median | 5 | 0 | 283.47299999999996 | 273.776 | 38.72481970907029 | [236.836, 339.515] | 0.0 |
| adaptive | diagnostics.pkt_belated_delta | 5 | 0 | 600.8 | 643.0 | 222.2548087218812 | [349.0, 842.0] | 0.0 |
| adaptive | diagnostics.pkt_belated_sum | 5 | 0 | 600.8 | 643.0 | 222.2548087218812 | [349.0, 842.0] | 0.0 |
| adaptive | diagnostics.pkt_drop_delta | 5 | 0 | 16524.0 | 16277.0 | 739.1745396048216 | [15816.0, 17369.0] | 0.0 |
| adaptive | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| adaptive | diagnostics.retrans_ratio | 5 | 0 | 0.18334749462296981 | 0.1836402538266006 | 0.003128519861894194 | [0.18014712729304405, 0.18753873637391083] | 0.0 |
| adaptive | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [34640.0, +inf] | 0.6 |
| adaptive | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [4906.0, +inf] | 0.6 |
| adaptive | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [34290.0, +inf] | 0.6 |
| adaptive | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [4794.0, +inf] | 0.6 |
| adaptive | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [33978.0, +inf] | 0.6 |
| adaptive | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [4687.0, +inf] | 0.6 |
| adaptive | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| adaptive | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| classic | useful_goodput_bps | 5 | 0 | 9115072.213333333 | 9143568.0 | 109883.6516635298 | [8928796.8, 9214982.933333334] | 0.0 |
| classic | viewer_loss_ratio | 5 | 0 | 0.24003317828237375 | 0.23807260412334383 | 0.009870578224378635 | [0.23143172616856827, 0.25696686113661266] | 0.0 |
| classic | per_link_share_gini | 5 | 0 | 0.021380086224210355 | 0.02065968117762018 | 0.008293082932534977 | [0.010065316188337614, 0.03323968547315315] | 0.0 |
| classic | cpu_ms_per_mb | 5 | 0 | 117.36389698846355 | 117.53252851257483 | 2.7750919082037466 | [113.44073786044484, 121.25560598116272] | 0.0 |
| classic | switch_count | 5 | 0 | 20192.8 | 20380.0 | 901.8024728287231 | [18817.0, 21171.0] | 0.0 |
| classic | switches_per_second | 5 | 0 | 336.54442992616794 | 339.66100564990586 | 15.030223579236027 | [313.6166666666667, 352.85] | 0.0 |
| classic | sender_cpu_percent | 5 | 0 | 13.36991077926479 | 13.433109448175864 | 0.2243442278482733 | [13.0, 13.55] | 0.0 |
| classic | diagnostics.loss_ratio | 5 | 0 | 0.4563144240482765 | 0.45351683783138286 | 0.007715060313521891 | [0.45172339127182537, 0.47004202521512906] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.607484784712142 | 10.614858653846152 | 0.11911254774698633 | [10.48107666666667, 10.790752452830183] | 0.0 |
| classic | diagnostics.ms_rcv_buf_min | 5 | 0 | 1814.2 | 1807.0 | 31.156058800817537 | [1784.0, 1861.0] | 0.0 |
| classic | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| classic | diagnostics.ms_rtt_median | 5 | 0 | 264.4105 | 253.2865 | 25.291163471655477 | [243.825, 306.983] | 0.0 |
| classic | diagnostics.pkt_belated_delta | 5 | 0 | 474.2 | 449.0 | 79.06136856897938 | [382.0, 578.0] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 5 | 0 | 474.2 | 449.0 | 79.06136856897938 | [382.0, 578.0] | 0.0 |
| classic | diagnostics.pkt_drop_delta | 5 | 0 | 16224.0 | 16010.0 | 644.5510065153882 | [15720.0, 17354.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 5 | 0 | 0.1737288376677138 | 0.171826453617399 | 0.00940232640809352 | [0.1629492746927839, 0.18863318134491797] | 0.0 |
| classic | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [34706.0, +inf] | 0.8 |
| classic | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [4891.0, +inf] | 0.8 |
| classic | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [34424.0, +inf] | 0.8 |
| classic | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [4780.0, +inf] | 0.8 |
| classic | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [34139.0, +inf] | 0.8 |
| classic | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [4671.0, +inf] | 0.8 |
| classic | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| classic | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| edpf | useful_goodput_bps | 5 | 0 | 8933604.586666666 | 8985472.533333333 | 196147.27162931216 | [8718236.8, 9175853.866666667] | 0.0 |
| edpf | viewer_loss_ratio | 5 | 0 | 0.2547835399314165 | 0.2501502161647248 | 0.016577097727978822 | [0.23483668009290565, 0.27249696287297404] | 0.0 |
| edpf | per_link_share_gini | 5 | 0 | 0.041649386489415775 | 0.02965217896366971 | 0.023754997067756158 | [0.022321748011728888, 0.07837933110649457] | 0.0 |
| edpf | cpu_ms_per_mb | 5 | 0 | 122.51509248456459 | 119.60046204359143 | 4.266386131464748 | [119.15330706225718, 127.39579024358073] | 0.0 |
| edpf | switch_count | 5 | 0 | 6647.2 | 6527.0 | 579.3256424499092 | [6101.0, 7624.0] | 0.0 |
| edpf | switches_per_second | 5 | 0 | 110.78551335255524 | 108.78152030799488 | 9.654659026660406 | [101.68333333333334, 127.0645489241846] | 0.0 |
| edpf | sender_cpu_percent | 5 | 0 | 13.673197280045333 | 13.666666666666666 | 0.2097482017117675 | [13.433109448175864, 13.883333333333333] | 0.0 |
| edpf | diagnostics.loss_ratio | 5 | 0 | 0.4601026387062067 | 0.45567933160930973 | 0.010170613628301505 | [0.4502573153923192, 0.4715554925928924] | 0.0 |
| edpf | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.60583859100145 | 10.622794615384612 | 0.17215045205853594 | [10.3965546, 10.83475] | 0.0 |
| edpf | diagnostics.ms_rcv_buf_min | 5 | 0 | 1739.8 | 1759.0 | 70.43223693735703 | [1666.0, 1817.0] | 0.0 |
| edpf | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| edpf | diagnostics.ms_rtt_median | 5 | 0 | 347.1513 | 378.005 | 61.703021039897244 | [271.86199999999997, 408.8825] | 0.0 |
| edpf | diagnostics.pkt_belated_delta | 5 | 0 | 987.2 | 1036.0 | 261.6767089368101 | [676.0, 1277.0] | 0.0 |
| edpf | diagnostics.pkt_belated_sum | 5 | 0 | 987.2 | 1036.0 | 261.6767089368101 | [676.0, 1277.0] | 0.0 |
| edpf | diagnostics.pkt_drop_delta | 5 | 0 | 17269.8 | 17069.0 | 1047.529092674757 | [15975.0, 18393.0] | 0.0 |
| edpf | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| edpf | diagnostics.retrans_ratio | 5 | 0 | 0.18736089802548986 | 0.1872163606073952 | 0.001995108144048917 | [0.1848846983152432, 0.19035056085941504] | 0.0 |
| edpf | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| edpf | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| enhanced | useful_goodput_bps | 5 | 0 | 8957432.959999999 | 8933709.866666667 | 46184.97994980172 | [8924761.066666666, 9036884.266666668] | 0.0 |
| enhanced | viewer_loss_ratio | 5 | 0 | 0.2528913905299275 | 0.2543583965844402 | 0.004455028674938355 | [0.24518486862464817, 0.2562331163367433] | 0.0 |
| enhanced | per_link_share_gini | 5 | 0 | 0.027462754015382906 | 0.022131359248632482 | 0.015887898440785562 | [0.01700183438117712, 0.055144542398848556] | 0.0 |
| enhanced | cpu_ms_per_mb | 5 | 0 | 116.69592020866682 | 117.90547813327987 | 4.97738208276778 | [108.16349324339707, 120.44978187395265] | 0.0 |
| enhanced | switch_count | 5 | 0 | 1083.2 | 1071.0 | 35.3864380801459 | [1056.0, 1145.0] | 0.0 |
| enhanced | switches_per_second | 5 | 0 | 18.05315555851847 | 17.85 | 0.589877373675994 | [17.599706671555474, 19.083333333333332] | 0.0 |
| enhanced | sender_cpu_percent | 5 | 0 | 13.066533946656444 | 13.216446392560124 | 0.5730651783997164 | [12.066666666666666, 13.45] | 0.0 |
| enhanced | diagnostics.loss_ratio | 5 | 0 | 0.45255753854851566 | 0.4514769218267512 | 0.0023552534181533876 | [0.45070897007931887, 0.4564256140209833] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.539115624434391 | 10.534329019607844 | 0.07161891359720973 | [10.43963576923077, 10.623657450980394] | 0.0 |
| enhanced | diagnostics.ms_rcv_buf_min | 5 | 0 | 1791.6 | 1803.0 | 18.22909761891685 | [1768.0, 1808.0] | 0.0 |
| enhanced | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 5 | 0 | 292.9672 | 285.829 | 19.653557431162437 | [278.598, 327.607] | 0.0 |
| enhanced | diagnostics.pkt_belated_delta | 5 | 0 | 682.6 | 607.0 | 314.3378755415898 | [422.0, 1213.0] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 5 | 0 | 682.6 | 607.0 | 314.3378755415898 | [422.0, 1213.0] | 0.0 |
| enhanced | diagnostics.pkt_drop_delta | 5 | 0 | 17082.6 | 17158.0 | 279.1429741189987 | [16638.0, 17358.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 5 | 0 | 0.1825337447988488 | 0.18317618783508963 | 0.008146247937708313 | [0.17006764268083582, 0.19279049100199955] | 0.0 |
| enhanced | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [34702.0, +inf] | 0.8 |
| enhanced | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [4885.0, +inf] | 0.8 |
| enhanced | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [34360.0, +inf] | 0.8 |
| enhanced | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [4776.0, +inf] | 0.8 |
| enhanced | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [34064.0, +inf] | 0.8 |
| enhanced | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [4665.0, +inf] | 0.8 |
| enhanced | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| enhanced | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |
| rtt-threshold | useful_goodput_bps | 5 | 0 | 9160061.866666667 | 9159360.0 | 79496.08300108548 | [9033901.333333334, 9229722.133333333] | 0.0 |
| rtt-threshold | viewer_loss_ratio | 5 | 0 | 0.23589492058323053 | 0.23722665715262603 | 0.00714946092917603 | [0.229002005426448, 0.24631606811965057] | 0.0 |
| rtt-threshold | per_link_share_gini | 5 | 0 | 0.027350208189789928 | 0.025919722701591413 | 0.008116204911472497 | [0.016159282398662083, 0.03611391503422412] | 0.0 |
| rtt-threshold | cpu_ms_per_mb | 5 | 0 | 112.62378280586306 | 115.17469530151405 | 6.064192681927013 | [104.22853416250335, 118.22134873880992] | 0.0 |
| rtt-threshold | switch_count | 5 | 0 | 991.6 | 996.0 | 42.78784874237077 | [942.0, 1056.0] | 0.0 |
| rtt-threshold | switches_per_second | 5 | 0 | 16.526552557457375 | 16.6 | 0.713010080899897 | [15.7, 17.599706671555474] | 0.0 |
| rtt-threshold | sender_cpu_percent | 5 | 0 | 12.893247168102755 | 13.283333333333331 | 0.6506972021086376 | [11.933333333333334, 13.4] | 0.0 |
| rtt-threshold | diagnostics.loss_ratio | 5 | 0 | 0.43658475899712756 | 0.4378826255934863 | 0.004234131132828783 | [0.4318702290076336, 0.4423020356827702] | 0.0 |
| rtt-threshold | diagnostics.mbps_recv_rate_mean | 5 | 0 | 10.821207203193032 | 10.7899848076923 | 0.09153055384027756 | [10.71873846153846, 10.92640245283019] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_buf_min | 5 | 0 | 1775.8 | 1759.0 | 40.57339029462537 | [1736.0, 1826.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rcv_tsbpd_delay | 5 | 0 | 2000.0 | 2000.0 | 0.0 | [2000.0, 2000.0] | 0.0 |
| rtt-threshold | diagnostics.ms_rtt_median | 5 | 0 | 293.2662 | 299.9705 | 33.229480786268695 | [247.312, 325.8775] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_delta | 5 | 0 | 975.6 | 978.0 | 189.1700293386878 | [713.0, 1219.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_belated_sum | 5 | 0 | 975.6 | 978.0 | 189.1700293386878 | [713.0, 1219.0] | 0.0 |
| rtt-threshold | diagnostics.pkt_drop_delta | 5 | 0 | 15957.0 | 15958.0 | 489.02249437014655 | [15530.0, 16749.0] | 0.0 |
| rtt-threshold | diagnostics.reorder_distance_max | 0 | 5 | None | None | None | [None, None] | None |
| rtt-threshold | diagnostics.retrans_ratio | 5 | 0 | 0.20232753846337065 | 0.20940849905951536 | 0.010880028646792038 | [0.1877637130801688, 0.21117923855696563] | 0.0 |
| rtt-threshold | episode[1].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[1].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[2].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[3].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[4].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[5].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].failover_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | episode[6].recovery_ms | 5 | 0 | +inf | +inf | None | [+inf, +inf] | 1.0 |
| rtt-threshold | load[0].reached_ms | 5 | 0 | 1000.0 | 1000.0 | 0.0 | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| adaptive | episode[1] graded=True | — | 0.4 | 0.6 |
| adaptive | episode[2] graded=True | — | 0.4 | 0.6 |
| adaptive | episode[3] graded=True | — | 0.4 | 0.6 |
| adaptive | episode[4] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[5] graded=True | — | 0.0 | 1.0 |
| adaptive | episode[6] graded=True | — | 0.0 | 1.0 |
| adaptive | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| adaptive | settled_rate | — | 1.0 | — |
| adaptive | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| adaptive | post_settle_starvation_floor_rate | — | 1.0 | — |
| classic | episode[1] graded=True | — | 0.2 | 0.8 |
| classic | episode[2] graded=True | — | 0.2 | 0.8 |
| classic | episode[3] graded=True | — | 0.2 | 0.8 |
| classic | episode[4] graded=True | — | 0.0 | 1.0 |
| classic | episode[5] graded=True | — | 0.0 | 1.0 |
| classic | episode[6] graded=True | — | 0.0 | 1.0 |
| classic | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| classic | settled_rate | — | 1.0 | — |
| classic | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| classic | post_settle_starvation_floor_rate | — | 1.0 | — |
| edpf | episode[1] graded=True | — | 0.0 | 1.0 |
| edpf | episode[2] graded=True | — | 0.0 | 1.0 |
| edpf | episode[3] graded=True | — | 0.0 | 1.0 |
| edpf | episode[4] graded=True | — | 0.0 | 1.0 |
| edpf | episode[5] graded=True | — | 0.0 | 1.0 |
| edpf | episode[6] graded=True | — | 0.0 | 1.0 |
| edpf | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| edpf | settled_rate | — | 1.0 | — |
| edpf | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| edpf | post_settle_starvation_floor_rate | — | 1.0 | — |
| enhanced | episode[1] graded=True | — | 0.2 | 0.8 |
| enhanced | episode[2] graded=True | — | 0.2 | 0.8 |
| enhanced | episode[3] graded=True | — | 0.2 | 0.8 |
| enhanced | episode[4] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[5] graded=True | — | 0.0 | 1.0 |
| enhanced | episode[6] graded=True | — | 0.0 | 1.0 |
| enhanced | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| enhanced | settled_rate | — | 1.0 | — |
| enhanced | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| enhanced | post_settle_starvation_floor_rate | — | 1.0 | — |
| rtt-threshold | episode[1] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[2] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[3] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[4] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[5] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | episode[6] graded=True | — | 0.0 | 1.0 |
| rtt-threshold | load[0] graded=True | 0.0 | 1.0 | 0.0 |
| rtt-threshold | settled_rate | — | 1.0 | — |
| rtt-threshold | post_settle_zero_drop_belated_rate | — | 0.0 | — |
| rtt-threshold | post_settle_starvation_floor_rate | — | 1.0 | — |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| adaptive | adaptive | 5 | [1.0, 1.0] | 0.0 | None | () |
| adaptive | classic | 5 | [0.9670963687948664, 1.0250953110875287] | 0.3375759717994725 | None | () |
| adaptive | edpf | 5 | [0.9844735565259582, 1.0376306340641486] | -0.8701852323386272 | None | () |
| adaptive | enhanced | 5 | [0.9861558774416527, 1.0251853016927825] | -1.2910032743101651 | None | () |
| adaptive | rtt-threshold | 5 | [0.9646653861514178, 1.0131688841410118] | 0.4221706688712523 | None | () |
| classic | adaptive | 5 | [0.9755190460671359, 1.0340231152414894] | -0.3375759717994725 | None | () |
| classic | classic | 5 | [1.0, 1.0] | 0.0 | None | () |
| classic | edpf | 5 | [0.9730753050063105, 1.0569778207140843] | -1.2077612041380998 | None | () |
| classic | enhanced | 5 | [0.9995089469859166, 1.0262010448992418] | -1.6285792461096378 | None | () |
| classic | rtt-threshold | 5 | [0.9881371076595503, 1.0070180821077257] | 0.0845946970717798 | None | () |
| edpf | adaptive | 5 | [0.9637340756635543, 1.0157713159191721] | 0.8701852323386272 | None | () |
| edpf | classic | 5 | [0.9460936458670526, 1.0276696930393427] | 1.2077612041380998 | None | () |
| edpf | edpf | 5 | [1.0, 1.0] | 0.0 | None | () |
| edpf | enhanced | 5 | [0.9647392334278281, 1.0271650527390939] | -0.42081804197153794 | None | () |
| edpf | rtt-threshold | 5 | [0.9527334087553452, 1.015713314557638] | 1.2923559012098795 | None | () |
| enhanced | adaptive | 5 | [0.9754334151580238, 1.0140384728976748] | 1.2910032743101651 | None | () |
| enhanced | classic | 5 | [0.9744679222171184, 1.0004912942656132] | 1.6285792461096378 | None | () |
| enhanced | edpf | 5 | [0.9735533713236699, 1.0365495310550257] | 0.42081804197153794 | None | () |
| enhanced | enhanced | 5 | [1.0, 1.0] | 0.0 | None | () |
| enhanced | rtt-threshold | 5 | [0.9669588030645806, 0.9888511216859279] | 1.7131739431814175 | None | () |
| rtt-threshold | adaptive | 5 | [0.9870022813105076, 1.036628881222277] | -0.4221706688712523 | None | () |
| rtt-threshold | classic | 5 | [0.9930308281128015, 1.0120053100409796] | -0.0845946970717798 | None | () |
| rtt-threshold | edpf | 5 | [0.9845297739702452, 1.0496115606005714] | -1.2923559012098795 | None | () |
| rtt-threshold | enhanced | 5 | [1.011274577203355, 1.0341702219688182] | -1.7131739431814175 | None | () |
| rtt-threshold | rtt-threshold | 5 | [1.0, 1.0] | 0.0 | None | () |

## Warnings

- m4a-ours-new/A/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/A/ours-new-200/production/adaptive run 0: loadavg>2 (11.75)
- m4a-ours-new/A/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/A/ours-new-200/production/adaptive run 1: loadavg>2 (8.65)
- m4a-ours-new/A/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/A/ours-new-200/production/adaptive run 2: loadavg>2 (9.85)
- m4a-ours-new/A/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/A/ours-new-200/production/adaptive run 3: loadavg>2 (8.04)
- m4a-ours-new/A/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/A/ours-new-200/production/adaptive run 4: loadavg>2 (14.95)
- m4a-ours-new/A/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/A/ours-new-200/production/classic run 0: loadavg>2 (9.44)
- m4a-ours-new/A/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/A/ours-new-200/production/classic run 1: loadavg>2 (9.45)
- m4a-ours-new/A/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/A/ours-new-200/production/classic run 2: loadavg>2 (7.79)
- m4a-ours-new/A/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/A/ours-new-200/production/classic run 3: loadavg>2 (8.16)
- m4a-ours-new/A/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/A/ours-new-200/production/classic run 4: loadavg>2 (10.02)
- m4a-ours-new/A/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/A/ours-new-200/production/edpf run 0: loadavg>2 (11.53)
- m4a-ours-new/A/ours-new-200/production/edpf run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/A/ours-new-200/production/edpf run 1: loadavg>2 (8.17)
- m4a-ours-new/A/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/A/ours-new-200/production/edpf run 2: loadavg>2 (7.27)
- m4a-ours-new/A/ours-new-200/production/edpf run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/A/ours-new-200/production/edpf run 3: loadavg>2 (9.21)
- m4a-ours-new/A/ours-new-200/production/edpf run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/A/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/A/ours-new-200/production/edpf run 4: loadavg>2 (9.77)
- m4a-ours-new/A/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/A/ours-new-200/production/enhanced run 0: loadavg>2 (9.79)
- m4a-ours-new/A/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/A/ours-new-200/production/enhanced run 1: loadavg>2 (8.35)
- m4a-ours-new/A/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/A/ours-new-200/production/enhanced run 2: loadavg>2 (9.99)
- m4a-ours-new/A/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/A/ours-new-200/production/enhanced run 3: loadavg>2 (8.8)
- m4a-ours-new/A/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/A/ours-new-200/production/enhanced run 4: loadavg>2 (9.1)
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 0: loadavg>2 (10.58)
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 1: loadavg>2 (8.41)
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 2: loadavg>2 (9.91)
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 3: loadavg>2 (7.72)
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/A/ours-new-200/production/rtt-threshold run 4: loadavg>2 (11.46)
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 0: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 0: loadavg>2 (9.31)
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 1: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 1: loadavg>2 (8.82)
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 2: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 2: loadavg>2 (10.89)
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 3: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 3: loadavg>2 (8.76)
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 4: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-classic run 4: loadavg>2 (6.66)
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 0: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 0: loadavg>2 (10.81)
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 1: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 1: loadavg>2 (7.51)
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 2: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 2: loadavg>2 (9.17)
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 3: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 3: loadavg>2 (7.59)
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 4: loadavg>2
- m4a-ours-new/A/ours-new-200/production/upstream-enhanced run 4: loadavg>2 (7.63)
- m4a-ours-new/B1/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/adaptive run 0: loadavg>2 (6.64)
- m4a-ours-new/B1/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/adaptive run 1: loadavg>2 (7.83)
- m4a-ours-new/B1/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/adaptive run 2: loadavg>2 (7.04)
- m4a-ours-new/B1/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/adaptive run 3: loadavg>2 (8.92)
- m4a-ours-new/B1/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/adaptive run 4: loadavg>2 (7.1)
- m4a-ours-new/B1/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/classic run 0: loadavg>2 (6.69)
- m4a-ours-new/B1/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/classic run 1: loadavg>2 (7.24)
- m4a-ours-new/B1/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/classic run 2: loadavg>2 (7.61)
- m4a-ours-new/B1/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/classic run 3: loadavg>2 (9.22)
- m4a-ours-new/B1/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/classic run 4: loadavg>2 (7.82)
- m4a-ours-new/B1/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/edpf run 0: loadavg>2 (7.05)
- m4a-ours-new/B1/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/edpf run 1: loadavg>2 (6.9)
- m4a-ours-new/B1/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/edpf run 2: loadavg>2 (7.08)
- m4a-ours-new/B1/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/edpf run 3: loadavg>2 (7.29)
- m4a-ours-new/B1/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/edpf run 4: loadavg>2 (6.95)
- m4a-ours-new/B1/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/enhanced run 0: loadavg>2 (7.89)
- m4a-ours-new/B1/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/enhanced run 1: loadavg>2 (8.91)
- m4a-ours-new/B1/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/enhanced run 2: loadavg>2 (6.54)
- m4a-ours-new/B1/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/enhanced run 3: loadavg>2 (8.25)
- m4a-ours-new/B1/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/enhanced run 4: loadavg>2 (8.06)
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 0: loadavg>2 (7.37)
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.99)
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 2: loadavg>2 (6.7)
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 3: loadavg>2 (7.76)
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/B1/ours-new-200/production/rtt-threshold run 4: loadavg>2 (7.85)
- m4a-ours-new/B2/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/adaptive run 0: loadavg>2 (6.45)
- m4a-ours-new/B2/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/adaptive run 1: loadavg>2 (9.12)
- m4a-ours-new/B2/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/adaptive run 2: loadavg>2 (7.08)
- m4a-ours-new/B2/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/adaptive run 3: loadavg>2 (8.65)
- m4a-ours-new/B2/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/adaptive run 4: loadavg>2 (8.25)
- m4a-ours-new/B2/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/classic run 0: loadavg>2 (6.18)
- m4a-ours-new/B2/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/classic run 1: loadavg>2 (7.64)
- m4a-ours-new/B2/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/classic run 2: loadavg>2 (7.13)
- m4a-ours-new/B2/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/classic run 3: loadavg>2 (6.69)
- m4a-ours-new/B2/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/classic run 4: loadavg>2 (9.08)
- m4a-ours-new/B2/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/edpf run 0: loadavg>2 (7.6)
- m4a-ours-new/B2/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/edpf run 1: loadavg>2 (6.46)
- m4a-ours-new/B2/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/edpf run 2: loadavg>2 (7.9)
- m4a-ours-new/B2/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/edpf run 3: loadavg>2 (7.84)
- m4a-ours-new/B2/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/edpf run 4: loadavg>2 (7.4)
- m4a-ours-new/B2/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/enhanced run 0: loadavg>2 (6.78)
- m4a-ours-new/B2/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/enhanced run 1: loadavg>2 (7.3)
- m4a-ours-new/B2/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/enhanced run 2: loadavg>2 (6.63)
- m4a-ours-new/B2/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/enhanced run 3: loadavg>2 (8.19)
- m4a-ours-new/B2/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/enhanced run 4: loadavg>2 (7.56)
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 0: loadavg>2 (7.07)
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 1: loadavg>2 (8.23)
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 2: loadavg>2 (6.22)
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 3: loadavg>2 (7.06)
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/B2/ours-new-200/production/rtt-threshold run 4: loadavg>2 (8.41)
- m4a-ours-new/C/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/C/ours-new-200/production/adaptive run 0: loadavg>2 (6.12)
- m4a-ours-new/C/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/C/ours-new-200/production/adaptive run 1: loadavg>2 (5.33)
- m4a-ours-new/C/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/C/ours-new-200/production/adaptive run 2: loadavg>2 (5.11)
- m4a-ours-new/C/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/C/ours-new-200/production/adaptive run 3: loadavg>2 (5.25)
- m4a-ours-new/C/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/C/ours-new-200/production/adaptive run 4: loadavg>2 (3.39)
- m4a-ours-new/C/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/C/ours-new-200/production/classic run 0: loadavg>2 (6.37)
- m4a-ours-new/C/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/C/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/C/ours-new-200/production/classic run 1: loadavg>2 (4.31)
- m4a-ours-new/C/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/C/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/C/ours-new-200/production/classic run 2: loadavg>2 (7.11)
- m4a-ours-new/C/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/C/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/C/ours-new-200/production/classic run 3: loadavg>2 (5.77)
- m4a-ours-new/C/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/C/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/C/ours-new-200/production/classic run 4: loadavg>2 (5.82)
- m4a-ours-new/C/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/C/ours-new-200/production/edpf run 0: loadavg>2 (10.67)
- m4a-ours-new/C/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/C/ours-new-200/production/edpf run 1: loadavg>2 (6.52)
- m4a-ours-new/C/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/C/ours-new-200/production/edpf run 2: loadavg>2 (3.24)
- m4a-ours-new/C/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/C/ours-new-200/production/edpf run 3: loadavg>2 (4.44)
- m4a-ours-new/C/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/C/ours-new-200/production/edpf run 4: loadavg>2 (5.32)
- m4a-ours-new/C/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/C/ours-new-200/production/enhanced run 0: loadavg>2 (7.71)
- m4a-ours-new/C/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/C/ours-new-200/production/enhanced run 1: loadavg>2 (6.08)
- m4a-ours-new/C/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/C/ours-new-200/production/enhanced run 2: loadavg>2 (5.19)
- m4a-ours-new/C/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/C/ours-new-200/production/enhanced run 3: loadavg>2 (4.22)
- m4a-ours-new/C/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/C/ours-new-200/production/enhanced run 4: loadavg>2 (6.0)
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 0: loadavg>2 (9.28)
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.29)
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.45)
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 3: loadavg>2 (3.73)
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/C/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.15)
- m4a-ours-new/D/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/D/ours-new-200/production/adaptive run 0: loadavg>2 (4.6)
- m4a-ours-new/D/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/D/ours-new-200/production/adaptive run 1: loadavg>2 (4.58)
- m4a-ours-new/D/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/D/ours-new-200/production/adaptive run 2: loadavg>2 (5.67)
- m4a-ours-new/D/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/D/ours-new-200/production/adaptive run 3: loadavg>2 (5.28)
- m4a-ours-new/D/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/D/ours-new-200/production/adaptive run 4: loadavg>2 (3.98)
- m4a-ours-new/D/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/D/ours-new-200/production/classic run 0: loadavg>2 (3.76)
- m4a-ours-new/D/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/D/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/D/ours-new-200/production/classic run 1: loadavg>2 (5.97)
- m4a-ours-new/D/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/D/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/D/ours-new-200/production/classic run 2: loadavg>2 (4.03)
- m4a-ours-new/D/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/D/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/D/ours-new-200/production/classic run 3: loadavg>2 (4.49)
- m4a-ours-new/D/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/D/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/D/ours-new-200/production/classic run 4: loadavg>2 (5.87)
- m4a-ours-new/D/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/D/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/D/ours-new-200/production/edpf run 0: loadavg>2 (3.08)
- m4a-ours-new/D/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/D/ours-new-200/production/edpf run 1: loadavg>2 (7.81)
- m4a-ours-new/D/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/D/ours-new-200/production/edpf run 2: loadavg>2 (4.11)
- m4a-ours-new/D/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/D/ours-new-200/production/edpf run 3: loadavg>2 (5.68)
- m4a-ours-new/D/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/D/ours-new-200/production/edpf run 4: loadavg>2 (6.08)
- m4a-ours-new/D/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/D/ours-new-200/production/enhanced run 0: loadavg>2 (3.4)
- m4a-ours-new/D/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/D/ours-new-200/production/enhanced run 1: loadavg>2 (5.02)
- m4a-ours-new/D/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/D/ours-new-200/production/enhanced run 2: loadavg>2 (4.05)
- m4a-ours-new/D/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/D/ours-new-200/production/enhanced run 3: loadavg>2 (7.64)
- m4a-ours-new/D/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/D/ours-new-200/production/enhanced run 4: loadavg>2 (6.17)
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 0: loadavg>2 (3.33)
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.88)
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 2: loadavg>2 (5.61)
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 3: loadavg>2 (4.24)
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/D/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.03)
- m4a-ours-new/E/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/E/ours-new-200/production/adaptive run 0: loadavg>2 (2.85)
- m4a-ours-new/E/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/E/ours-new-200/production/adaptive run 1: loadavg>2 (3.48)
- m4a-ours-new/E/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/E/ours-new-200/production/adaptive run 2: loadavg>2 (4.45)
- m4a-ours-new/E/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/E/ours-new-200/production/adaptive run 3: loadavg>2 (4.47)
- m4a-ours-new/E/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/E/ours-new-200/production/adaptive run 4: loadavg>2 (4.48)
- m4a-ours-new/E/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/E/ours-new-200/production/classic run 0: loadavg>2 (3.7)
- m4a-ours-new/E/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/E/ours-new-200/production/classic run 1: loadavg>2 (3.84)
- m4a-ours-new/E/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/E/ours-new-200/production/classic run 2: loadavg>2 (5.19)
- m4a-ours-new/E/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/E/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/E/ours-new-200/production/classic run 3: loadavg>2 (6.09)
- m4a-ours-new/E/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/E/ours-new-200/production/classic run 4: loadavg>2 (4.4)
- m4a-ours-new/E/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/E/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/E/ours-new-200/production/edpf run 0: loadavg>2 (3.3)
- m4a-ours-new/E/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/E/ours-new-200/production/edpf run 1: loadavg>2 (3.59)
- m4a-ours-new/E/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/E/ours-new-200/production/edpf run 2: loadavg>2 (4.66)
- m4a-ours-new/E/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/E/ours-new-200/production/edpf run 3: loadavg>2 (5.53)
- m4a-ours-new/E/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/E/ours-new-200/production/edpf run 4: loadavg>2 (9.65)
- m4a-ours-new/E/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/E/ours-new-200/production/enhanced run 0: loadavg>2 (4.13)
- m4a-ours-new/E/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/E/ours-new-200/production/enhanced run 1: loadavg>2 (3.72)
- m4a-ours-new/E/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/E/ours-new-200/production/enhanced run 2: loadavg>2 (4.92)
- m4a-ours-new/E/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/E/ours-new-200/production/enhanced run 3: loadavg>2 (4.73)
- m4a-ours-new/E/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/E/ours-new-200/production/enhanced run 4: loadavg>2 (3.19)
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.0)
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 1: loadavg>2 (3.97)
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 2: loadavg>2 (5.23)
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 3: loadavg>2 (3.84)
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/E/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.1)
- m4a-ours-new/F/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/F/ours-new-200/production/adaptive run 0: loadavg>2 (5.04)
- m4a-ours-new/F/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/F/ours-new-200/production/adaptive run 1: loadavg>2 (3.8)
- m4a-ours-new/F/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/F/ours-new-200/production/adaptive run 2: loadavg>2 (2.7)
- m4a-ours-new/F/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/F/ours-new-200/production/adaptive run 3: loadavg>2 (5.14)
- m4a-ours-new/F/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/F/ours-new-200/production/adaptive run 4: loadavg>2 (4.33)
- m4a-ours-new/F/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/F/ours-new-200/production/classic run 0: loadavg>2 (4.97)
- m4a-ours-new/F/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/F/ours-new-200/production/classic run 1: loadavg>2 (5.25)
- m4a-ours-new/F/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/F/ours-new-200/production/classic run 2: loadavg>2 (3.41)
- m4a-ours-new/F/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/F/ours-new-200/production/classic run 3: loadavg>2 (3.83)
- m4a-ours-new/F/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/F/ours-new-200/production/classic run 4: loadavg>2 (3.25)
- m4a-ours-new/F/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/F/ours-new-200/production/edpf run 0: loadavg>2 (4.82)
- m4a-ours-new/F/ours-new-200/production/edpf run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/F/ours-new-200/production/edpf run 1: loadavg>2 (3.8)
- m4a-ours-new/F/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/F/ours-new-200/production/edpf run 2: loadavg>2 (4.33)
- m4a-ours-new/F/ours-new-200/production/edpf run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/F/ours-new-200/production/edpf run 3: loadavg>2 (3.29)
- m4a-ours-new/F/ours-new-200/production/edpf run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/F/ours-new-200/production/edpf run 4: loadavg>2 (2.9)
- m4a-ours-new/F/ours-new-200/production/edpf run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/F/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/F/ours-new-200/production/enhanced run 0: loadavg>2 (4.21)
- m4a-ours-new/F/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/F/ours-new-200/production/enhanced run 1: loadavg>2 (4.44)
- m4a-ours-new/F/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/F/ours-new-200/production/enhanced run 2: loadavg>2 (3.06)
- m4a-ours-new/F/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/F/ours-new-200/production/enhanced run 3: loadavg>2 (3.56)
- m4a-ours-new/F/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/F/ours-new-200/production/enhanced run 4: loadavg>2 (3.24)
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 0: loadavg>2 (7.98)
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 1: loadavg>2 (4.92)
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.37)
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 3: loadavg>2 (3.95)
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/F/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.3)
- m4a-ours-new/G/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/G/ours-new-200/production/adaptive run 0: loadavg>2 (4.12)
- m4a-ours-new/G/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/G/ours-new-200/production/adaptive run 1: loadavg>2 (4.44)
- m4a-ours-new/G/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/G/ours-new-200/production/adaptive run 2: loadavg>2 (4.8)
- m4a-ours-new/G/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/G/ours-new-200/production/adaptive run 3: loadavg>2 (4.08)
- m4a-ours-new/G/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/G/ours-new-200/production/adaptive run 4: loadavg>2 (3.49)
- m4a-ours-new/G/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/G/ours-new-200/production/classic run 0: loadavg>2 (3.23)
- m4a-ours-new/G/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/G/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/G/ours-new-200/production/classic run 1: loadavg>2 (4.45)
- m4a-ours-new/G/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/G/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/G/ours-new-200/production/classic run 2: loadavg>2 (6.29)
- m4a-ours-new/G/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/G/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/G/ours-new-200/production/classic run 3: loadavg>2 (4.84)
- m4a-ours-new/G/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/G/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/G/ours-new-200/production/classic run 4: loadavg>2 (4.08)
- m4a-ours-new/G/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/G/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/G/ours-new-200/production/edpf run 0: loadavg>2 (3.72)
- m4a-ours-new/G/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/G/ours-new-200/production/edpf run 1: loadavg>2 (6.85)
- m4a-ours-new/G/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/G/ours-new-200/production/edpf run 2: loadavg>2 (5.27)
- m4a-ours-new/G/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/G/ours-new-200/production/edpf run 3: loadavg>2 (4.07)
- m4a-ours-new/G/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/G/ours-new-200/production/edpf run 4: loadavg>2 (4.12)
- m4a-ours-new/G/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/G/ours-new-200/production/enhanced run 0: loadavg>2 (3.57)
- m4a-ours-new/G/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/G/ours-new-200/production/enhanced run 1: loadavg>2 (5.81)
- m4a-ours-new/G/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/G/ours-new-200/production/enhanced run 2: loadavg>2 (5.51)
- m4a-ours-new/G/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/G/ours-new-200/production/enhanced run 3: loadavg>2 (4.6)
- m4a-ours-new/G/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/G/ours-new-200/production/enhanced run 4: loadavg>2 (4.08)
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.03)
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 1: loadavg>2 (3.46)
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 2: loadavg>2 (5.17)
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 3: loadavg>2 (4.69)
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/G/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.91)
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 0: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 0: loadavg>2 (3.72)
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 1: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 1: loadavg>2 (4.92)
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 2: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 2: loadavg>2 (3.95)
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 3: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 3: loadavg>2 (5.18)
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 4: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-classic run 4: loadavg>2 (4.93)
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 0: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 0: loadavg>2 (3.51)
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 1: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 1: loadavg>2 (4.09)
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 2: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 2: loadavg>2 (4.54)
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 3: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 3: loadavg>2 (5.4)
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 4: loadavg>2
- m4a-ours-new/G/ours-new-200/production/upstream-enhanced run 4: loadavg>2 (6.04)
- m4a-ours-new/H/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/H/ours-new-200/production/adaptive run 0: loadavg>2 (4.66)
- m4a-ours-new/H/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/H/ours-new-200/production/adaptive run 1: loadavg>2 (5.11)
- m4a-ours-new/H/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/H/ours-new-200/production/adaptive run 2: loadavg>2 (35.65)
- m4a-ours-new/H/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/H/ours-new-200/production/adaptive run 3: loadavg>2 (4.94)
- m4a-ours-new/H/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/H/ours-new-200/production/adaptive run 4: loadavg>2 (7.39)
- m4a-ours-new/H/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/H/ours-new-200/production/classic run 0: loadavg>2 (3.99)
- m4a-ours-new/H/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/H/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/H/ours-new-200/production/classic run 1: loadavg>2 (5.64)
- m4a-ours-new/H/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/H/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/H/ours-new-200/production/classic run 2: loadavg>2 (8.75)
- m4a-ours-new/H/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/H/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/H/ours-new-200/production/classic run 3: loadavg>2 (4.94)
- m4a-ours-new/H/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/H/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/H/ours-new-200/production/classic run 4: loadavg>2 (4.12)
- m4a-ours-new/H/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/H/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/H/ours-new-200/production/edpf run 0: loadavg>2 (5.39)
- m4a-ours-new/H/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/H/ours-new-200/production/edpf run 1: loadavg>2 (4.75)
- m4a-ours-new/H/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/H/ours-new-200/production/edpf run 2: loadavg>2 (24.33)
- m4a-ours-new/H/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/H/ours-new-200/production/edpf run 3: loadavg>2 (4.04)
- m4a-ours-new/H/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/H/ours-new-200/production/edpf run 4: loadavg>2 (4.84)
- m4a-ours-new/H/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/H/ours-new-200/production/enhanced run 0: loadavg>2 (5.21)
- m4a-ours-new/H/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/H/ours-new-200/production/enhanced run 1: loadavg>2 (4.35)
- m4a-ours-new/H/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/H/ours-new-200/production/enhanced run 2: loadavg>2 (35.85)
- m4a-ours-new/H/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/H/ours-new-200/production/enhanced run 3: loadavg>2 (3.92)
- m4a-ours-new/H/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/H/ours-new-200/production/enhanced run 4: loadavg>2 (6.84)
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 0: loadavg>2 (6.39)
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 1: loadavg>2 (3.35)
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 2: loadavg>2 (31.56)
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 3: loadavg>2 (5.31)
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/H/ours-new-200/production/rtt-threshold run 4: loadavg>2 (5.23)
- m4a-ours-new/I/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/I/ours-new-200/production/adaptive run 0: loadavg>2 (4.0)
- m4a-ours-new/I/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/I/ours-new-200/production/adaptive run 1: loadavg>2 (4.4)
- m4a-ours-new/I/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/I/ours-new-200/production/adaptive run 2: loadavg>2 (4.19)
- m4a-ours-new/I/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/I/ours-new-200/production/adaptive run 3: loadavg>2 (5.74)
- m4a-ours-new/I/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/I/ours-new-200/production/adaptive run 4: loadavg>2 (4.6)
- m4a-ours-new/I/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/I/ours-new-200/production/classic run 0: loadavg>2 (5.29)
- m4a-ours-new/I/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/I/ours-new-200/production/classic run 1: loadavg>2 (3.75)
- m4a-ours-new/I/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/I/ours-new-200/production/classic run 2: loadavg>2 (4.41)
- m4a-ours-new/I/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/I/ours-new-200/production/classic run 3: loadavg>2 (5.34)
- m4a-ours-new/I/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/I/ours-new-200/production/classic run 4: loadavg>2 (3.97)
- m4a-ours-new/I/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/I/ours-new-200/production/edpf run 0: loadavg>2 (3.87)
- m4a-ours-new/I/ours-new-200/production/edpf run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/I/ours-new-200/production/edpf run 1: loadavg>2 (3.43)
- m4a-ours-new/I/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/I/ours-new-200/production/edpf run 2: loadavg>2 (5.73)
- m4a-ours-new/I/ours-new-200/production/edpf run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/I/ours-new-200/production/edpf run 3: loadavg>2 (3.77)
- m4a-ours-new/I/ours-new-200/production/edpf run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/I/ours-new-200/production/edpf run 4: loadavg>2 (4.29)
- m4a-ours-new/I/ours-new-200/production/edpf run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/I/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/I/ours-new-200/production/enhanced run 0: loadavg>2 (5.52)
- m4a-ours-new/I/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/I/ours-new-200/production/enhanced run 1: loadavg>2 (5.24)
- m4a-ours-new/I/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/I/ours-new-200/production/enhanced run 2: loadavg>2 (4.33)
- m4a-ours-new/I/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/I/ours-new-200/production/enhanced run 3: loadavg>2 (5.29)
- m4a-ours-new/I/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/I/ours-new-200/production/enhanced run 4: loadavg>2 (4.24)
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 0: loadavg>2 (3.9)
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.19)
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.08)
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 3: loadavg>2 (3.78)
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/I/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.52)
- m4a-ours-new/J/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/J/ours-new-200/production/adaptive run 0: loadavg>2 (5.21)
- m4a-ours-new/J/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/J/ours-new-200/production/adaptive run 1: loadavg>2 (6.59)
- m4a-ours-new/J/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/J/ours-new-200/production/adaptive run 2: loadavg>2 (8.64)
- m4a-ours-new/J/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/J/ours-new-200/production/adaptive run 3: loadavg>2 (4.98)
- m4a-ours-new/J/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/J/ours-new-200/production/adaptive run 4: loadavg>2 (8.27)
- m4a-ours-new/J/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/J/ours-new-200/production/classic run 0: loadavg>2 (4.21)
- m4a-ours-new/J/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/J/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/J/ours-new-200/production/classic run 1: loadavg>2 (9.62)
- m4a-ours-new/J/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/J/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/J/ours-new-200/production/classic run 2: loadavg>2 (6.29)
- m4a-ours-new/J/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/J/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/J/ours-new-200/production/classic run 3: loadavg>2 (5.34)
- m4a-ours-new/J/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/J/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/J/ours-new-200/production/classic run 4: loadavg>2 (5.93)
- m4a-ours-new/J/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/J/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/J/ours-new-200/production/edpf run 0: loadavg>2 (4.14)
- m4a-ours-new/J/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/J/ours-new-200/production/edpf run 1: loadavg>2 (9.95)
- m4a-ours-new/J/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/J/ours-new-200/production/edpf run 2: loadavg>2 (10.26)
- m4a-ours-new/J/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/J/ours-new-200/production/edpf run 3: loadavg>2 (5.0)
- m4a-ours-new/J/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/J/ours-new-200/production/edpf run 4: loadavg>2 (5.22)
- m4a-ours-new/J/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/J/ours-new-200/production/enhanced run 0: loadavg>2 (4.37)
- m4a-ours-new/J/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/J/ours-new-200/production/enhanced run 1: loadavg>2 (7.58)
- m4a-ours-new/J/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/J/ours-new-200/production/enhanced run 2: loadavg>2 (4.63)
- m4a-ours-new/J/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/J/ours-new-200/production/enhanced run 3: loadavg>2 (6.11)
- m4a-ours-new/J/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/J/ours-new-200/production/enhanced run 4: loadavg>2 (21.56)
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.62)
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.17)
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.64)
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 3: loadavg>2 (4.3)
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/J/ours-new-200/production/rtt-threshold run 4: loadavg>2 (15.93)
- m4a-ours-new/K/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/K/ours-new-200/production/adaptive run 0: loadavg>2 (5.46)
- m4a-ours-new/K/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/K/ours-new-200/production/adaptive run 1: loadavg>2 (5.45)
- m4a-ours-new/K/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/K/ours-new-200/production/adaptive run 2: loadavg>2 (4.95)
- m4a-ours-new/K/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/K/ours-new-200/production/adaptive run 3: loadavg>2 (4.88)
- m4a-ours-new/K/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/K/ours-new-200/production/adaptive run 4: loadavg>2 (4.32)
- m4a-ours-new/K/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/K/ours-new-200/production/classic run 0: loadavg>2 (8.8)
- m4a-ours-new/K/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/K/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/K/ours-new-200/production/classic run 1: loadavg>2 (4.89)
- m4a-ours-new/K/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/K/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/K/ours-new-200/production/classic run 2: loadavg>2 (5.91)
- m4a-ours-new/K/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/K/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/K/ours-new-200/production/classic run 3: loadavg>2 (4.55)
- m4a-ours-new/K/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/K/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/K/ours-new-200/production/classic run 4: loadavg>2 (4.12)
- m4a-ours-new/K/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/K/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/K/ours-new-200/production/edpf run 0: loadavg>2 (5.49)
- m4a-ours-new/K/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/K/ours-new-200/production/edpf run 1: loadavg>2 (3.74)
- m4a-ours-new/K/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/K/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/K/ours-new-200/production/edpf run 2: loadavg>2 (12.46)
- m4a-ours-new/K/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/K/ours-new-200/production/edpf run 3: loadavg>2 (5.03)
- m4a-ours-new/K/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/K/ours-new-200/production/edpf run 4: loadavg>2 (4.11)
- m4a-ours-new/K/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/K/ours-new-200/production/enhanced run 0: loadavg>2 (5.16)
- m4a-ours-new/K/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/K/ours-new-200/production/enhanced run 1: loadavg>2 (3.5)
- m4a-ours-new/K/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/K/ours-new-200/production/enhanced run 2: loadavg>2 (5.34)
- m4a-ours-new/K/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/K/ours-new-200/production/enhanced run 3: loadavg>2 (3.95)
- m4a-ours-new/K/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/K/ours-new-200/production/enhanced run 4: loadavg>2 (4.88)
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 0: loadavg>2 (6.91)
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.5)
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.51)
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 3: loadavg>2 (5.57)
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/K/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.94)
- m4a-ours-new/L/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/L/ours-new-200/production/adaptive run 0: loadavg>2 (4.35)
- m4a-ours-new/L/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/L/ours-new-200/production/adaptive run 1: loadavg>2 (3.69)
- m4a-ours-new/L/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/L/ours-new-200/production/adaptive run 2: loadavg>2 (6.17)
- m4a-ours-new/L/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/L/ours-new-200/production/adaptive run 3: loadavg>2 (4.18)
- m4a-ours-new/L/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/L/ours-new-200/production/adaptive run 4: loadavg>2 (3.33)
- m4a-ours-new/L/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/L/ours-new-200/production/classic run 0: loadavg>2 (5.62)
- m4a-ours-new/L/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/L/ours-new-200/production/classic run 1: loadavg>2 (5.43)
- m4a-ours-new/L/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/L/ours-new-200/production/classic run 2: loadavg>2 (6.83)
- m4a-ours-new/L/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/L/ours-new-200/production/classic run 3: loadavg>2 (5.43)
- m4a-ours-new/L/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/L/ours-new-200/production/classic run 4: loadavg>2 (5.08)
- m4a-ours-new/L/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/L/ours-new-200/production/edpf run 0: loadavg>2 (4.46)
- m4a-ours-new/L/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/L/ours-new-200/production/edpf run 1: loadavg>2 (5.27)
- m4a-ours-new/L/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/L/ours-new-200/production/edpf run 2: loadavg>2 (5.03)
- m4a-ours-new/L/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/L/ours-new-200/production/edpf run 3: loadavg>2 (4.62)
- m4a-ours-new/L/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/L/ours-new-200/production/edpf run 4: loadavg>2 (4.58)
- m4a-ours-new/L/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/L/ours-new-200/production/enhanced run 0: loadavg>2 (4.78)
- m4a-ours-new/L/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/L/ours-new-200/production/enhanced run 1: loadavg>2 (5.33)
- m4a-ours-new/L/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/L/ours-new-200/production/enhanced run 2: loadavg>2 (3.8)
- m4a-ours-new/L/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/L/ours-new-200/production/enhanced run 3: loadavg>2 (6.29)
- m4a-ours-new/L/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/L/ours-new-200/production/enhanced run 4: loadavg>2 (3.72)
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.68)
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 1: loadavg>2 (4.35)
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.37)
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 3: loadavg>2 (4.78)
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/L/ours-new-200/production/rtt-threshold run 4: loadavg>2 (5.04)
- m4a-ours-new/M1/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/adaptive run 0: loadavg>2 (5.07)
- m4a-ours-new/M1/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/adaptive run 1: loadavg>2 (5.21)
- m4a-ours-new/M1/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/adaptive run 2: loadavg>2 (3.93)
- m4a-ours-new/M1/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/adaptive run 3: loadavg>2 (4.01)
- m4a-ours-new/M1/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/adaptive run 4: loadavg>2 (3.14)
- m4a-ours-new/M1/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/classic run 0: loadavg>2 (5.29)
- m4a-ours-new/M1/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/classic run 1: loadavg>2 (4.05)
- m4a-ours-new/M1/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M1/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/classic run 2: loadavg>2 (2.92)
- m4a-ours-new/M1/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M1/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/classic run 3: loadavg>2 (3.51)
- m4a-ours-new/M1/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M1/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/classic run 4: loadavg>2 (4.52)
- m4a-ours-new/M1/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/edpf run 0: loadavg>2 (4.24)
- m4a-ours-new/M1/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/edpf run 1: loadavg>2 (4.39)
- m4a-ours-new/M1/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/edpf run 2: loadavg>2 (9.24)
- m4a-ours-new/M1/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/edpf run 3: loadavg>2 (3.51)
- m4a-ours-new/M1/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/edpf run 4: loadavg>2 (3.64)
- m4a-ours-new/M1/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/enhanced run 0: loadavg>2 (3.07)
- m4a-ours-new/M1/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/enhanced run 1: loadavg>2 (4.04)
- m4a-ours-new/M1/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/enhanced run 2: loadavg>2 (3.6)
- m4a-ours-new/M1/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/enhanced run 3: loadavg>2 (3.84)
- m4a-ours-new/M1/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/enhanced run 4: loadavg>2 (2.25)
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.13)
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 1: loadavg>2 (4.37)
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.07)
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 3: loadavg>2 (3.72)
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/rtt-threshold run 4: loadavg>2 (2.64)
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 0: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 0: loadavg>2 (3.92)
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 1: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 1: loadavg>2 (4.44)
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 2: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 2: loadavg>2 (3.09)
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 3: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 3: loadavg>2 (5.91)
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 4: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-classic run 4: loadavg>2 (4.76)
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 0: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 0: loadavg>2 (2.21)
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 1: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 1: loadavg>2 (4.49)
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 2: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 2: loadavg>2 (3.69)
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 3: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 3: loadavg>2 (3.77)
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 4: loadavg>2
- m4a-ours-new/M1/ours-new-200/production/upstream-enhanced run 4: loadavg>2 (4.32)
- m4a-ours-new/M2/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/adaptive run 0: loadavg>2 (5.18)
- m4a-ours-new/M2/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/adaptive run 1: loadavg>2 (5.27)
- m4a-ours-new/M2/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/adaptive run 2: loadavg>2 (4.7)
- m4a-ours-new/M2/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/adaptive run 3: loadavg>2 (4.68)
- m4a-ours-new/M2/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/adaptive run 4: loadavg>2 (4.36)
- m4a-ours-new/M2/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/classic run 0: loadavg>2 (5.1)
- m4a-ours-new/M2/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M2/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/classic run 1: loadavg>2 (5.12)
- m4a-ours-new/M2/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M2/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/classic run 2: loadavg>2 (5.67)
- m4a-ours-new/M2/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M2/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/classic run 3: loadavg>2 (5.17)
- m4a-ours-new/M2/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M2/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/classic run 4: loadavg>2 (5.77)
- m4a-ours-new/M2/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M2/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/edpf run 0: loadavg>2 (4.88)
- m4a-ours-new/M2/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/edpf run 1: loadavg>2 (3.08)
- m4a-ours-new/M2/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/edpf run 2: loadavg>2 (5.65)
- m4a-ours-new/M2/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/edpf run 3: loadavg>2 (7.2)
- m4a-ours-new/M2/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/edpf run 4: loadavg>2 (8.36)
- m4a-ours-new/M2/ours-new-200/production/edpf run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M2/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/enhanced run 0: loadavg>2 (4.05)
- m4a-ours-new/M2/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/enhanced run 1: loadavg>2 (4.59)
- m4a-ours-new/M2/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/enhanced run 2: loadavg>2 (6.28)
- m4a-ours-new/M2/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/enhanced run 3: loadavg>2 (4.59)
- m4a-ours-new/M2/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/enhanced run 4: loadavg>2 (13.37)
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.0)
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 1: loadavg>2 (4.94)
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 2: loadavg>2 (5.89)
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 3: loadavg>2 (4.92)
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/M2/ours-new-200/production/rtt-threshold run 4: loadavg>2 (8.64)
- m4a-ours-new/M3/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/adaptive run 0: loadavg>2 (9.81)
- m4a-ours-new/M3/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/adaptive run 1: loadavg>2 (5.24)
- m4a-ours-new/M3/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/adaptive run 2: loadavg>2 (4.47)
- m4a-ours-new/M3/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/adaptive run 3: loadavg>2 (4.69)
- m4a-ours-new/M3/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/adaptive run 4: loadavg>2 (3.7)
- m4a-ours-new/M3/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/classic run 0: loadavg>2 (4.9)
- m4a-ours-new/M3/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/classic run 1: loadavg>2 (4.92)
- m4a-ours-new/M3/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/classic run 2: loadavg>2 (6.72)
- m4a-ours-new/M3/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/classic run 3: loadavg>2 (3.67)
- m4a-ours-new/M3/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/classic run 4: loadavg>2 (4.12)
- m4a-ours-new/M3/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/edpf run 0: loadavg>2 (2.7)
- m4a-ours-new/M3/ours-new-200/production/edpf run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/edpf run 1: loadavg>2 (5.53)
- m4a-ours-new/M3/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/edpf run 2: loadavg>2 (4.24)
- m4a-ours-new/M3/ours-new-200/production/edpf run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/edpf run 3: loadavg>2 (4.6)
- m4a-ours-new/M3/ours-new-200/production/edpf run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/edpf run 4: loadavg>2 (5.48)
- m4a-ours-new/M3/ours-new-200/production/edpf run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M3/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/enhanced run 0: loadavg>2 (3.31)
- m4a-ours-new/M3/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/enhanced run 1: loadavg>2 (4.57)
- m4a-ours-new/M3/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/enhanced run 2: loadavg>2 (4.06)
- m4a-ours-new/M3/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/enhanced run 3: loadavg>2 (5.29)
- m4a-ours-new/M3/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/enhanced run 4: loadavg>2 (5.42)
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 0: loadavg>2 (7.23)
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.89)
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.48)
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 3: loadavg>2 (4.94)
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/M3/ours-new-200/production/rtt-threshold run 4: loadavg>2 (5.3)
- m4a-ours-new/M4/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/adaptive run 0: loadavg>2 (3.87)
- m4a-ours-new/M4/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/adaptive run 1: loadavg>2 (4.88)
- m4a-ours-new/M4/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/adaptive run 2: loadavg>2 (4.45)
- m4a-ours-new/M4/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/adaptive run 3: loadavg>2 (3.56)
- m4a-ours-new/M4/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/adaptive run 4: loadavg>2 (4.36)
- m4a-ours-new/M4/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/classic run 0: loadavg>2 (5.38)
- m4a-ours-new/M4/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/classic run 1: loadavg>2 (5.95)
- m4a-ours-new/M4/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/classic run 2: loadavg>2 (5.17)
- m4a-ours-new/M4/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/classic run 3: loadavg>2 (2.91)
- m4a-ours-new/M4/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/classic run 4: loadavg>2 (3.64)
- m4a-ours-new/M4/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/edpf run 0: loadavg>2 (6.03)
- m4a-ours-new/M4/ours-new-200/production/edpf run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/edpf run 1: loadavg>2 (5.58)
- m4a-ours-new/M4/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/edpf run 2: loadavg>2 (7.18)
- m4a-ours-new/M4/ours-new-200/production/edpf run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/edpf run 3: loadavg>2 (4.89)
- m4a-ours-new/M4/ours-new-200/production/edpf run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/edpf run 4: loadavg>2 (4.53)
- m4a-ours-new/M4/ours-new-200/production/edpf run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/enhanced run 0: loadavg>2 (2.58)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 0: loadavg>2 (3.13)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 0: loadavg>2 (3.59)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/enhanced run 1: loadavg>2 (3.3)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 1: loadavg>2 (3.54)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 1: loadavg>2 (4.91)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/enhanced run 2: loadavg>2 (2.72)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 2: loadavg>2 (4.77)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 2: loadavg>2 (5.63)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/enhanced run 3: loadavg>2 (4.12)
- m4a-ours-new/M4/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/enhanced run 4: loadavg>2 (4.37)
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.65)
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.56)
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.73)
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 3: loadavg>2 (3.79)
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.56)
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 0: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 0: loadavg>2 (5.25)
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 1: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 1: loadavg>2 (5.64)
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 2: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 2: loadavg>2 (6.2)
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 3: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 3: loadavg>2 (3.97)
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 4: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-classic run 4: loadavg>2 (2.83)
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 0: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 0: loadavg>2 (5.37)
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 1: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 1: loadavg>2 (3.9)
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 2: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 2: loadavg>2 (4.09)
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 3: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 3: loadavg>2 (4.6)
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 4: loadavg>2
- m4a-ours-new/M4/ours-new-200/production/upstream-enhanced run 4: loadavg>2 (2.73)
- m4a-ours-new/M5/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/adaptive run 0: loadavg>2 (2.53)
- m4a-ours-new/M5/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/adaptive run 1: loadavg>2 (3.87)
- m4a-ours-new/M5/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/adaptive run 2: loadavg>2 (2.97)
- m4a-ours-new/M5/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/adaptive run 3: loadavg>2 (3.62)
- m4a-ours-new/M5/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/adaptive run 4: loadavg>2 (3.78)
- m4a-ours-new/M5/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/classic run 0: loadavg>2 (3.56)
- m4a-ours-new/M5/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/classic run 1: loadavg>2 (5.09)
- m4a-ours-new/M5/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/classic run 2: loadavg>2 (2.45)
- m4a-ours-new/M5/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/classic run 3: loadavg>2 (2.91)
- m4a-ours-new/M5/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/classic run 4: loadavg>2 (4.54)
- m4a-ours-new/M5/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/edpf run 0: loadavg>2 (2.74)
- m4a-ours-new/M5/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/edpf run 1: loadavg>2 (3.93)
- m4a-ours-new/M5/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/edpf run 2: loadavg>2 (3.25)
- m4a-ours-new/M5/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/edpf run 3: loadavg>2 (2.92)
- m4a-ours-new/M5/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/edpf run 4: loadavg>2 (2.56)
- m4a-ours-new/M5/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/enhanced run 0: loadavg>2 (2.65)
- m4a-ours-new/M5/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/enhanced run 1: loadavg>2 (2.62)
- m4a-ours-new/M5/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/enhanced run 2: loadavg>2 (3.45)
- m4a-ours-new/M5/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/enhanced run 3: loadavg>2 (5.04)
- m4a-ours-new/M5/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/enhanced run 4: loadavg>2 (3.25)
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.13)
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 1: loadavg>2 (3.95)
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 2: loadavg>2 (3.51)
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 3: loadavg>2 (2.92)
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/M5/ours-new-200/production/rtt-threshold run 4: loadavg>2 (6.29)
- m4a-ours-new/M6/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/adaptive run 0: loadavg>2 (3.88)
- m4a-ours-new/M6/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/adaptive run 1: loadavg>2 (2.33)
- m4a-ours-new/M6/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/adaptive run 2: loadavg>2 (2.19)
- m4a-ours-new/M6/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/adaptive run 3: loadavg>2 (3.51)
- m4a-ours-new/M6/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/adaptive run 4: loadavg>2 (3.69)
- m4a-ours-new/M6/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/classic run 0: loadavg>2 (3.25)
- m4a-ours-new/M6/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/classic run 1: loadavg>2 (3.08)
- m4a-ours-new/M6/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/classic run 2: loadavg>2 (3.47)
- m4a-ours-new/M6/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/classic run 3: loadavg>2 (4.3)
- m4a-ours-new/M6/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/classic run 4: loadavg>2 (3.52)
- m4a-ours-new/M6/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/edpf run 0: loadavg>2 (2.74)
- m4a-ours-new/M6/ours-new-200/production/edpf run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/edpf run 1: loadavg>2 (3.17)
- m4a-ours-new/M6/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/edpf run 2: loadavg>2 (2.09)
- m4a-ours-new/M6/ours-new-200/production/edpf run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/edpf run 3: loadavg>2 (3.84)
- m4a-ours-new/M6/ours-new-200/production/edpf run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/edpf run 4: loadavg>2 (3.72)
- m4a-ours-new/M6/ours-new-200/production/edpf run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M6/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/enhanced run 0: loadavg>2 (3.71)
- m4a-ours-new/M6/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/enhanced run 1: loadavg>2 (3.85)
- m4a-ours-new/M6/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/enhanced run 2: loadavg>2 (3.25)
- m4a-ours-new/M6/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/enhanced run 3: loadavg>2 (3.0)
- m4a-ours-new/M6/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/enhanced run 4: loadavg>2 (2.71)
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 0: loadavg>2 (4.84)
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 1: loadavg>2 (3.48)
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 2: loadavg>2 (2.85)
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 3: loadavg>2 (3.69)
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/M6/ours-new-200/production/rtt-threshold run 4: loadavg>2 (3.98)
- m4a-ours-new/M7/ours-new-200/production/adaptive run 0: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/adaptive run 0: loadavg>2 (2.79)
- m4a-ours-new/M7/ours-new-200/production/adaptive run 1: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/adaptive run 1: loadavg>2 (4.38)
- m4a-ours-new/M7/ours-new-200/production/adaptive run 2: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/adaptive run 2: loadavg>2 (3.84)
- m4a-ours-new/M7/ours-new-200/production/adaptive run 3: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/adaptive run 3: loadavg>2 (3.59)
- m4a-ours-new/M7/ours-new-200/production/adaptive run 4: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/adaptive run 4: loadavg>2 (5.0)
- m4a-ours-new/M7/ours-new-200/production/classic run 0: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/classic run 0: loadavg>2 (2.98)
- m4a-ours-new/M7/ours-new-200/production/classic run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/classic run 1: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/classic run 1: loadavg>2 (4.82)
- m4a-ours-new/M7/ours-new-200/production/classic run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/classic run 2: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/classic run 2: loadavg>2 (5.36)
- m4a-ours-new/M7/ours-new-200/production/classic run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/classic run 3: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/classic run 3: loadavg>2 (3.61)
- m4a-ours-new/M7/ours-new-200/production/classic run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/classic run 4: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/classic run 4: loadavg>2 (2.63)
- m4a-ours-new/M7/ours-new-200/production/classic run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/edpf run 0: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/edpf run 0: loadavg>2 (3.41)
- m4a-ours-new/M7/ours-new-200/production/edpf run 0: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/edpf run 1: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/edpf run 1: loadavg>2 (3.78)
- m4a-ours-new/M7/ours-new-200/production/edpf run 1: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/edpf run 2: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/edpf run 2: loadavg>2 (3.07)
- m4a-ours-new/M7/ours-new-200/production/edpf run 2: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/edpf run 3: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/edpf run 3: loadavg>2 (4.14)
- m4a-ours-new/M7/ours-new-200/production/edpf run 3: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/edpf run 4: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/edpf run 4: loadavg>2 (4.33)
- m4a-ours-new/M7/ours-new-200/production/edpf run 4: switch churn DEFECT signal (informational, MIN_SWITCH_INTERVAL_MS=15)
- m4a-ours-new/M7/ours-new-200/production/enhanced run 0: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/enhanced run 0: loadavg>2 (4.23)
- m4a-ours-new/M7/ours-new-200/production/enhanced run 1: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/enhanced run 1: loadavg>2 (4.33)
- m4a-ours-new/M7/ours-new-200/production/enhanced run 2: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/enhanced run 2: loadavg>2 (3.38)
- m4a-ours-new/M7/ours-new-200/production/enhanced run 3: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/enhanced run 3: loadavg>2 (2.65)
- m4a-ours-new/M7/ours-new-200/production/enhanced run 4: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/enhanced run 4: loadavg>2 (2.79)
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 0: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 0: loadavg>2 (3.76)
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 1: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 1: loadavg>2 (5.85)
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 2: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 2: loadavg>2 (4.98)
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 3: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 3: loadavg>2 (2.97)
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 4: loadavg>2
- m4a-ours-new/M7/ours-new-200/production/rtt-threshold run 4: loadavg>2 (4.67)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off run 3: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off run 4: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 3: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 4: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B2@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B2@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B2@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B2@--production--slt:4001--fec:off run 3: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B2@--production--slt:4001--fec:off run 4: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 3: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 4: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--D@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--D@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--D@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--D@--production--slt:4001--fec:off run 3: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--D@--production--slt:4001--fec:off run 4: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off run 3: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off run 4: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--H@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--L@--production--slt:4001--fec:off run 3: failed (settle_timeout)
