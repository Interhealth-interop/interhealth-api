CREATE OR REPLACE VIEW "DBAMV"."VITAL_SIGNS"
(
    "vital_signs_code",
    "vital_signs_patient_code",
    "vital_signs_encounter_code",
    "vital_signs_class",
    "vital_signs_category",
    "vital_signs_value",
    "vital_signs_measure_value",
    "vital_signs_type",
    "vitalsigns_provider_code",
    "vital_signs_date",
    "vital_signs_time",
    "vital_signs_update_date",
    "vital_signs_update_time",
    "vital_signs_reason",
    "vital_signs_responsible"
) AS
SELECT CSV.CD_COLETA_SINAL_VITAL AS "vital_signs_code",
       A.CD_PACIENTE AS "vital_signs_patient_code",
       CSV.CD_ATENDIMENTO AS "vital_signs_encounter_code",
       SV.DS_SINAL_VITAL AS "vital_signs_class",
       SV.DS_SINAL_VITAL as "vital_signs_category",                                                                           
       ISV.VALOR AS "vital_signs_value",
       PUF.DS_UNIDADE_AFERICAO AS "vital_signs_measure_value",
       NULL AS "vital_signs_type",
       CSV.CD_PRESTADOR AS "vitalsigns_provider_code",
       TO_CHAR(CSV.DATA_COLETA, 'YYYY-MM-DD') AS "vital_signs_date",  
       TO_CHAR(CSV.DATA_COLETA, 'HH24:MI:SS') AS "vital_signs_time",
       NULL AS "vital_signs_update_date",
       NULL AS "vital_signs_update_time",
       NULL AS "vital_signs_reason",
       NULL AS "vital_signs_responsible"
 
 FROM DBAMV.COLETA_SINAL_VITAL CSV
 INNER JOIN DBAMV.ATENDIME A ON CSV.CD_ATENDIMENTO = A.CD_ATENDIMENTO
 INNER JOIN DBAMV.ITCOLETA_SINAL_VITAL ISV ON CSV.CD_COLETA_SINAL_VITAL = ISV.CD_COLETA_SINAL_VITAL
 LEFT JOIN DBAMV.PW_UNIDADE_AFERICAO PUF ON ISV.CD_UNIDADE_AFERICAO = PUF.CD_UNIDADE_AFERICAO
 INNER JOIN DBAMV.SINAL_VITAL SV ON ISV.CD_SINAL_VITAL = SV.CD_SINAL_VITAL
 WHERE CSV.SN_FINALIZADO = 'S';
